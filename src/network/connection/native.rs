use std::io::{Write, Read};
use anyhow::{Result, anyhow};
use winit::event_loop::EventLoopProxy;
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::{channel, Sender};
use log::error;

use crate::app::UVxlEvent;
use crate::game::network::packet::ServerPacket;

pub struct Connection {
  sender: Sender<Vec<u8>>,
}

impl Connection {
  pub fn new(address: SocketAddr, callback: EventLoopProxy<UVxlEvent>) -> Result<Self> {
    let mut socket = TcpStream::connect(address)?;
    socket.set_nonblocking(false)?;
    let (sender, receiver) = channel::<Vec<u8>>();

    let callback_packet = callback.clone();
    let send_packet_event = move |packet: ServerPacket| {
      // yeah, whatever
      unsafe impl Send for UVxlEvent {}
      callback_packet.send_event(UVxlEvent::IncomingPacket(packet))
        .map_err(|_| anyhow!("Failed to propagate incoming packet")).unwrap();
    };

    let callback_open = callback.clone();
    let send_open_event = move || {
      callback_open.send_event(UVxlEvent::ConnectionReady)
        .map_err(|_| anyhow!("Failed to send connection open event")).unwrap();
    };

    let socket_send = socket.try_clone()?;
    std::thread::spawn(move || {
      let mut socket = socket_send;
      let mut buffer = vec![0u8; 1024 * 1024 * 2];
      loop {
        'a: {
          let mut length = [0u8; 4];
          let Ok(()) = socket.read_exact(&mut length) else { break 'a; };
          let length = u32::from_be_bytes([length[0], length[1], length[2], length[3]]) as usize;

          let buffer = &mut buffer[.. length];
          let Ok(()) = socket.read_exact(buffer) else { break 'a; };

          let Ok(packet) = bincode::deserialize::<ServerPacket>(buffer) else { break 'a; };
          send_packet_event(packet);
        }
      }
    });

    std::thread::spawn(move || {
      loop {
        if let Ok(packet) = receiver.recv() {
          while let Err(err) = socket.write_all(&packet) {
            error!("An error has occurred while sending a packet: {}", err);
          }
        };
      }
    });

    send_open_event();
    return Ok(Self { sender });
  }

  pub fn send(&mut self, packet: impl serde::Serialize) -> Result<()> {
    let data = bincode::serialize(&packet)?;
    self.sender.send(data)?;

    return Ok(());
  }
}