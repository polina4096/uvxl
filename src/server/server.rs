use std::{net::SocketAddr, sync::Mutex};

use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};

use tap::Tap;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Runtime;
use tokio_tungstenite::tungstenite::protocol::Message;

use anyhow::Result;
use dashmap::DashMap;
use glam::{IVec3, Vec3};
use log::info;
use tokio_util::bytes::Bytes;
use tokio_util::codec::{BytesCodec, FramedRead, FramedWrite, LengthDelimitedCodec};
use crate::game::entity::Entity;
use crate::game::network::packet::{ClientPacket, InitialChunkDataServerPacket, ClientJoinServerPacket, ServerPacket, ClientMovePacket};
use crate::game::player::Player;
use crate::game::world::BlockId;
use crate::game::world::chunk::{Chunk, ChunkVec3Ext};
use crate::game::world::chunk_manager::ChunkManager;
use crate::game::world::world::World;

fn generate_chunk(color: bool) -> Chunk {
  return Chunk { blocks: vec![BlockId::AIR; 32 * 32 * 32] }
    .tap_mut(|chunk| {
      for x in 0 .. 32 {
        for y in 0 .. 12 {
          for z in 0 .. 32 {
            let y = y
              + (((x as f32 / 4.0).sin() + 1.0) * 2.0).round() as usize
              + (((z as f32 / 4.0).cos() + 1.0) * 2.0).round() as usize;

            if color {
              chunk.blocks[x + y * 32 + z * 32 * 32] = BlockId::TEST;
            } else {
              chunk.blocks[x + y * 32 + z * 32 * 32] = BlockId::PANEL;
            }

          }
        }
      }
    });
}

type Tx = UnboundedSender<Message>;

struct ServerPlayer {
  tx         : Tx,
  player     : Player,
  last_chunk : IVec3,
}

impl Default for ServerPlayer {
  fn default() -> Self {
    return Self {
      tx         : unbounded().0,
      player     : Player::default(),
      last_chunk : IVec3::MAX,
    };
  }
}

pub struct Server {
  peers: DashMap<SocketAddr, ServerPlayer>,
  world: Mutex<World>,
}

impl Server {
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    let world = World {
      chunk_manager: ChunkManager {
        chunks: Default::default()
      }
    };

    return Self {
      peers: DashMap::new(),
      world: Mutex::new(world),
    };
  }

  pub fn run(&'static self, address: SocketAddr) -> Result<()> {
    let rt = Runtime::new()?;

    let (ws_listener, tcp_listener) = rt.block_on(async {
      let address_ws = address.tap_mut(|x| x.set_port(x.port() + 1));

      // Create the event loop and TCP listener we'll accept connections on.
      let ws_socket = TcpListener::bind(&address_ws).await;
      let ws_listener = ws_socket.expect("Failed to bind");
      println!("WebSocket on: {}", &address_ws);

      let tcp_socket = TcpListener::bind(&address).await;
      let tcp_listener = tcp_socket.expect("Failed to bind");
      println!("TCP on: {}", &address);

      return (ws_listener, tcp_listener);
    });

    rt.spawn(async move {
      let ws_listener = ws_listener;
      while let Ok((stream, addr)) = ws_listener.accept().await
        { tokio::spawn(handle_ws_connection(self, stream, addr)); }
    });

    rt.block_on(async move {
      let tcp_listener = tcp_listener;
      while let Ok((stream, addr)) = tcp_listener.accept().await
        { tokio::spawn(handle_tcp_connection(self, stream, addr)); }
    });

    return Ok(());
  }

  pub fn handle_packet(&self, packet: &[u8], peer_addr: SocketAddr) -> Result<()> {
    let packet = match bincode::deserialize::<ClientPacket>(packet) {
      Ok(packet) => packet,
      Err(err) => {
        println!("Failed to deserialize packet: {:?}", err);
        return Ok(());
      }
    };

    match packet {
      ClientPacket::ClientJoinClientPacket(packet) => {
        for mut peer in self.peers.iter_mut() {
          let addr = peer.key();
          if *addr != peer_addr {
            // notify others about the new player

          } else {
            peer.player.name = packet.name.clone();
            let state = peer.player.entity.state_mut();
            state.position = Vec3::new(16.0, 12.0, 16.0);
            state.title = Some(packet.name.clone());

            // respond to the client
            let packet = bincode::serialize(&ServerPacket::ClientJoinServerPacket(ClientJoinServerPacket {
              success: true,
              reason: None,
            }))?;

            peer.tx.unbounded_send(Message::Binary(packet))?;
          }
        }
      }

      ClientPacket::ClientMovePacket(ClientMovePacket { position }) => {
        let mut peer = self.peers.get_mut(&peer_addr).unwrap();
        let state = peer.player.entity.state_mut();
        state.position = position;

        let chunk_pos = position.to_chunk_pos();
        let chunk_delta = peer.last_chunk - chunk_pos;

        let vertical_render_distance = 4;
        let horizontal_render_distance = 2;
        #[allow(clippy::match_single_binding)]
        match chunk_delta.to_array() {
          [dx, dy, dz] if dx != 0 => {
            peer.last_chunk = chunk_pos;

            let chunk_manager = &mut self.world.lock().unwrap().chunk_manager;

            let dx_capped = dx.abs().min(horizontal_render_distance * 2);
            let offset = if dx.abs() > horizontal_render_distance
                 { -dx_capped / 2 }
            else { horizontal_render_distance - dx.abs() + 1 };

            for x in 0 ..= dx_capped {
              for y in -vertical_render_distance ..= vertical_render_distance {
                for z in -horizontal_render_distance..= horizontal_render_distance {
                  let chunk_pos = IVec3::new(chunk_pos.x - (x + offset) * (dx / dx.abs()), chunk_pos.y , chunk_pos.z + z);
                  let chunk = chunk_manager.chunks.get(&chunk_pos).cloned()
                    .unwrap_or_else(|| {
                      let chunk = generate_chunk((chunk_pos.x + chunk_pos.z) % 2 == 0);
                      chunk_manager.chunks.insert(chunk_pos, chunk.clone());
                      return chunk.clone();
                    });

                  let packet = bincode::serialize(&ServerPacket::InitialChunkDataServerPacket(InitialChunkDataServerPacket {
                    chunk: chunk,
                    position: IVec3::new(chunk_pos.x, chunk_pos.y, chunk_pos.z),
                  }))?;

                  peer.tx.unbounded_send(Message::Binary(packet))?;
                }
              }
            }

            info!("{} moved to {:?} @ {:?}", peer.player.name, position, chunk_pos);
          }

          [dx, dy, dz] if dy != 0 => {
          }

          [dx, dy, dz] if dz != 0 => {
            peer.last_chunk = chunk_pos;

            let chunk_manager = &mut self.world.lock().unwrap().chunk_manager;

            let dz_capped = dz.abs().min(horizontal_render_distance * 2);
            let offset = if dz.abs() > horizontal_render_distance { -dz_capped / 2 }
            else { horizontal_render_distance - dz.abs() + 1 };

            for x in -horizontal_render_distance..=horizontal_render_distance {
              for y in -vertical_render_distance ..= vertical_render_distance {
                for z in 0 ..= dz_capped {
                  let chunk_pos = IVec3::new(chunk_pos.x + x, chunk_pos.y, chunk_pos.z - (z + offset) * (dz / dz.abs()));
                  let chunk = chunk_manager.chunks.get(&chunk_pos).cloned()
                    .unwrap_or_else(|| {
                      let chunk = generate_chunk((chunk_pos.x + chunk_pos.z) % 2 == 0);
                      chunk_manager.chunks.insert(chunk_pos, chunk.clone());
                      return chunk.clone();
                    });

                  let packet = bincode::serialize(&ServerPacket::InitialChunkDataServerPacket(InitialChunkDataServerPacket {
                    chunk: chunk,
                    position: IVec3::new(chunk_pos.x, chunk_pos.y, chunk_pos.z),
                  }))?;

                  peer.tx.unbounded_send(Message::Binary(packet))?;
                }
              }
            }

            info!("{} moved to {:?} @ {:?}", peer.player.name, position, chunk_pos);
          }

          _ => { }
        }
      }
    }

    return Ok(());
  }

  pub fn broadcast(&self, packet: ServerPacket) {
    for peer in self.peers.iter() {
      peer.value().tx.unbounded_send(Message::Binary(bincode::serialize(&packet).unwrap())).unwrap();
    }
  }

  pub fn broadcast_except(&self, packet: ServerPacket, except: SocketAddr) {
    for peer in self.peers.iter() {
      if peer.key() != &except {
        peer.value().tx.unbounded_send(Message::Binary(bincode::serialize(&packet).unwrap())).unwrap();
      }
    }
  }
}

async fn handle_tcp_connection(server: &Server, mut raw_stream: TcpStream, addr: SocketAddr) {
  println!("Incoming TCP connection from: {}", addr);

  // Insert the write part of this peer to the peer map.
  let (tx, rx) = unbounded();
  server.peers.insert(addr, ServerPlayer {
    tx,
    .. Default::default()
  });

  let (incoming, outgoing) = raw_stream.split();
  let outgoing = FramedWrite::new(outgoing, LengthDelimitedCodec::new());
  let incoming = FramedRead::new(incoming, BytesCodec::new());

  let broadcast_incoming = incoming.try_for_each(|msg| {
    server.handle_packet(&msg, addr).unwrap();

    // We want to broadcast the message to everyone except ourselves.
    // let peers = peer_map.lock().unwrap();
    // let broadcast_recipients = peers.iter()
    //   .filter(|(peer_addr, _)| peer_addr != &&addr)
    //   .map(|(_, ws_sink)| ws_sink);
    //
    // for recp in broadcast_recipients {
    //   recp.unbounded_send(Message::Binary(msg.to_vec())).unwrap();
    // }

    return future::ok(());
  });

  let receive_from_others = rx
    .map(|x| Bytes::from(x.into_data()))
    .map(Ok)
    .forward(outgoing);

  pin_mut!(broadcast_incoming, receive_from_others);
  future::select(broadcast_incoming, receive_from_others).await;

  println!("{} disconnected", &addr);
  server.peers.remove(&addr);
}

async fn handle_ws_connection(server: &Server, raw_stream: TcpStream, addr: SocketAddr) {
  println!("Incoming TCP connection from: {}", addr);

  let ws_stream = tokio_tungstenite::accept_async(raw_stream).await
    .expect("Error during the websocket handshake occurred");

  println!("WebSocket connection established: {}", addr);

  // Insert the write part of this peer to the peer map.
  let (tx, rx) = unbounded();
  server.peers.insert(addr, ServerPlayer {
    tx,
    .. Default::default()
  });

  let (outgoing, incoming) = ws_stream.split();

  let broadcast_incoming = incoming.try_for_each(|msg| {
    server.handle_packet(&msg.into_data(), addr).unwrap();

    return future::ok(());
  });

  let receive_from_others = rx
    .map(Ok)
    .forward(outgoing);

  pin_mut!(broadcast_incoming, receive_from_others);
  future::select(broadcast_incoming, receive_from_others).await;

  println!("{} disconnected", &addr);
  server.peers.remove(&addr);
}