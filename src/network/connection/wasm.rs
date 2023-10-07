use anyhow::{Result, anyhow};
use wasm_bindgen::{JsValue, prelude::Closure, JsCast};
use web_sys::Event;
use winit::event_loop::EventLoopProxy;
use std::net::SocketAddr;

use web_sys::{ErrorEvent, WebSocket, MessageEvent};

use crate::{game::network::packet::ServerPacket, app::UVxlEvent};

pub struct Connection {
  socket: WebSocket,
}

trait JsValueExt<T> {
  fn to_err(self) -> Result<T, anyhow::Error>;
}
impl<T> JsValueExt<T> for Result<T, JsValue> {
    fn to_err(self) -> Result<T, anyhow::Error> { self.map_err(|err| { web_sys::console::log_1(&err); anyhow!("an error occurred") }) }
}

impl Connection {
  pub fn new(address: SocketAddr, callback: EventLoopProxy<UVxlEvent>) -> Result<Self> {
    let socket = WebSocket::new(&format!("ws://{}", address)).to_err()?;
    socket.set_binary_type(web_sys::BinaryType::Arraybuffer);

    let callback_packet = callback.clone();
    let send_packet_event = move |packet: ServerPacket| {
      callback_packet.send_event(UVxlEvent::IncomingPacket(packet))
        .map_err(|_| anyhow!("Failed to propagate incoming packet")).unwrap();
    };

    let callback_open = callback.clone();
    let send_open_event = move || {
      callback_open.send_event(UVxlEvent::ConnectionReady)
        .map_err(|_| anyhow!("Failed to send connection open event")).unwrap();
    };

    // open callback
    let onopen_callback = Closure::<dyn FnMut(_)>::new(move |e: Event| {
      send_open_event();
    });

    socket.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();

    // message callback
    let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
      if let Ok(abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
        let array = js_sys::Uint8Array::new(&abuf);
        let len = array.byte_length() as usize;

        let data = array.to_vec();
        let Ok(packet) = bincode::deserialize::<ServerPacket>(&data) else { todo!() };
        send_packet_event(packet);
      }
    });

    socket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget();

    // error callback
    let onerror_callback = Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
      log::warn!("error event: {:?}", e.message());
    });

    socket.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    return Ok(Self { socket });
  }

  pub fn send(&mut self, packet: impl serde::Serialize) -> Result<()> {
    let data = bincode::serialize(&packet)?;
    self.socket.send_with_u8_array(&data).to_err()?;

    return Ok(());
  }
}