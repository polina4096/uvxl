use std::io::ErrorKind;
use std::net::SocketAddr;
use std::sync::atomic::Ordering;

use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};

use tap::Tap;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Runtime;
use tokio_tungstenite::tungstenite::protocol::Message;

use anyhow::{anyhow, Result};
use dashmap::DashMap;
use futures_channel::mpsc::unbounded;
use glam::{IVec3, ivec3, vec3};
use log::{error, info};
use tokio_util::bytes::Bytes;
use tokio_util::codec::{BytesCodec, FramedRead, FramedWrite, LengthDelimitedCodec};
use uuid::Uuid;
use crate::game::entity::Entity;
use crate::game::network::packet::{ClientPacket, InitialChunkDataServerPacket, ClientJoinSuccessServerPacket, ServerPacket, ClientMovePacket, PlayerJoinServerPacket, PlayerMoveServerPacket, InitialPlayerData, ErrorServerPacket, ServerError};
use crate::game::world::chunk::ChunkVec3Ext;
use crate::game::world::worldgen::worldgen::WorldGen;
use crate::server::player::{ServerPlayer, Tx};
use crate::server::server_settings::ServerSettings;
use crate::server::world::chunk_manager::ServerChunkManager;
use crate::server::world::world::ServerWorld;

pub struct Server {
  peers    : DashMap<SocketAddr, ServerPlayer>,
  world    : ServerWorld,
  settings : ServerSettings,
  worldgen : WorldGen,
}

impl Server {
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    let world = ServerWorld {
      chunk_manager: ServerChunkManager {
        chunks: Default::default()
      }
    };

    let settings = ServerSettings {
      vertical_render_distance   : 3.into(),
      horizontal_render_distance : 2.into(),
    };

    let worldgen = WorldGen {

    };

    return Self {
      peers: DashMap::new(),
      world,
      settings,
      worldgen,
    };
  }

  pub fn run(&'static self, address: SocketAddr) -> Result<()> {
    let rt = Runtime::new()?;

    let (ws_listener, tcp_listener) = rt.block_on(async {
      let address_ws = address.tap_mut(|x| x.set_port(x.port() + 1));

      // Create the event loop and TCP listener we'll accept connections on.
      let ws_socket = TcpListener::bind(&address_ws).await;
      let ws_listener = ws_socket.expect("Failed to bind");
      info!("WebSocket on: {}", &address_ws);

      let tcp_socket = TcpListener::bind(&address).await;
      let tcp_listener = tcp_socket.expect("Failed to bind");
      info!("TCP on: {}", &address);

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
        error!("Failed to deserialize packet: {:?}", err);
        return Ok(());
      }
    };

    match packet {
      ClientPacket::ClientJoinClientPacket(packet) => {
        if self.peers.iter().filter(|peer| *peer.key() != peer_addr).any(|peer| peer.player.name == packet.name) {
          error!("Player with name {} is already connected to the server", packet.name);

          let packet = bincode::serialize(&ServerPacket::ErrorServerPacket(ErrorServerPacket {
            error: ServerError::PlayerLoggedIn,
          }))?;

          self.peers.get(&peer_addr).unwrap().tx.unbounded_send(Message::Binary(packet))?;
          return Err(anyhow!(""));
        }

        let uuid = Uuid::new_v4();
        let players_data = self.peers.iter()
          .filter(|x| *x.key() != peer_addr)
          .map(|x| InitialPlayerData {
            uuid: x.player.uuid,
            name: x.player.name.clone(),
            position: x.player.entity.state().position,
          })
          .collect::<Vec<_>>();

        for mut peer in self.peers.iter_mut() {
          // notify others about the new player
          if *peer.key() != peer_addr {
            let packet = bincode::serialize(&ServerPacket::PlayerJoinServerPacket(PlayerJoinServerPacket {
              name: packet.name.clone(),
              uuid: uuid,
              position: vec3(0.0, 0.0, 0.0),
            }))?;

            peer.tx.unbounded_send(Message::Binary(packet))?;
          }

          // respond to the client
          else {
            // process player info
            let player = &mut peer.player;
            player.uuid = uuid;
            player.name = packet.name.clone();

            let position = vec3(16.0, 34.0, 16.0);
            let state = player.entity.state_mut();
            state.position = position;
            state.title    = Some(packet.name.clone());

            let packet = bincode::serialize(&ServerPacket::ClientJoinSuccessServerPacket(ClientJoinSuccessServerPacket {
              uuid,
              position,
              players: players_data.clone(),
            }))?;

            peer.tx.unbounded_send(Message::Binary(packet))?;

            // send initial chunks
            let chunk_pos = position.to_chunk_pos();
            let chunk_manager = &self.world.chunk_manager;
            let vertical_render_distance = self.settings.vertical_render_distance.load(Ordering::Relaxed) as i32;
            let horizontal_render_distance = self.settings.horizontal_render_distance.load(Ordering::Relaxed) as i32;

            for x in -horizontal_render_distance ..= horizontal_render_distance {
              for y in -vertical_render_distance ..= vertical_render_distance {
                for z in -horizontal_render_distance ..= horizontal_render_distance {
                  let chunk_pos = ivec3(
                    chunk_pos.x + x,
                    chunk_pos.y + y,
                    chunk_pos.z + z,
                  );

                  let chunk = chunk_manager.chunks.get(&chunk_pos).map(|x| x.clone())
                    .unwrap_or_else(|| {
                      let chunk = self.worldgen.generate(chunk_pos);
                      chunk_manager.chunks.insert(chunk_pos, chunk.clone());
                      return chunk;
                    });

                  let packet = bincode::serialize(&ServerPacket::InitialChunkDataServerPacket(InitialChunkDataServerPacket {
                    chunk: chunk,
                    position: ivec3(chunk_pos.x, chunk_pos.y, chunk_pos.z),
                  }))?;

                  peer.tx.unbounded_send(Message::Binary(packet))?;
                }
              }
            }
          }
        }
      }

      ClientPacket::ClientMovePacket(ClientMovePacket { position }) => {
        let uuid = self.peers.get(&peer_addr).unwrap().player.uuid;
        for mut peer in self.peers.iter_mut() {
          // notify others about player movement2
          if *peer.key() != peer_addr {
            let packet = bincode::serialize(&ServerPacket::PlayerMoveServerPacket(PlayerMoveServerPacket {
              uuid,
              position,
            }))?;

            peer.tx.unbounded_send(Message::Binary(packet))?;
          }

          // respond to the client
          else {
            let state = peer.player.entity.state_mut();
            state.position = position;

            // send new chunks
            let chunk_pos = position.to_chunk_pos();
            let chunk_delta = peer.last_chunk - chunk_pos;

            let vertical_render_distance = self.settings.vertical_render_distance.load(Ordering::Relaxed) as i32;
            let horizontal_render_distance = self.settings.horizontal_render_distance.load(Ordering::Relaxed) as i32;

            let chunk_manager = &self.world.chunk_manager;
            let send_chunk = |chunk_pos: IVec3, tx: &Tx| -> Result<()> {
              let chunk = chunk_manager.chunks.get(&chunk_pos).map(|x| x.clone())
                .unwrap_or_else(|| {
                  let chunk = self.worldgen.generate(chunk_pos);
                  chunk_manager.chunks.insert(chunk_pos, chunk.clone());
                  return chunk;
                });

              let packet = bincode::serialize(&ServerPacket::InitialChunkDataServerPacket(InitialChunkDataServerPacket {
                chunk: chunk,
                position: ivec3(chunk_pos.x, chunk_pos.y, chunk_pos.z),
              }))?;

              tx.unbounded_send(Message::Binary(packet))?;

              return Ok(());
            };

            match chunk_delta.to_array() {
              [dx, dy, dz] if dx != 0 || dy != 0 || dz != 0 => {
                peer.last_chunk = chunk_pos;

                let dx_capped = dx.abs().min(horizontal_render_distance * 2);
                let dz_capped = dz.abs().min(horizontal_render_distance * 2);
                let dy_capped = dy.abs().min(vertical_render_distance * 2);
                let x_offset = if dx.abs() > horizontal_render_distance { -dx_capped / 2 } else { horizontal_render_distance - dx.abs() + 1 };
                let z_offset = if dz.abs() > horizontal_render_distance { -dz_capped / 2 } else { horizontal_render_distance - dz.abs() + 1 };
                let y_offset = if dy.abs() > vertical_render_distance { -dy_capped / 2 } else { vertical_render_distance - dy.abs() + 1 };

                for x in if dx != 0 { 0 ..= dx_capped } else { -horizontal_render_distance ..= horizontal_render_distance } {
                  for y in if dy != 0 { 0 ..= dy_capped } else { -vertical_render_distance ..= vertical_render_distance } {
                    for z in if dz != 0 { 0 ..= dz_capped } else { -horizontal_render_distance ..= horizontal_render_distance } {
                      let chunk_pos = ivec3(
                        chunk_pos.x - if dx != 0 { (x + x_offset) * (dx / dx.abs()) } else { -x },
                        chunk_pos.y - if dy != 0 { (y + y_offset) * (dy / dy.abs()) } else { -y },
                        chunk_pos.z - if dz != 0 { (z + z_offset) * (dz / dz.abs()) } else { -z }
                      );

                      send_chunk(chunk_pos, &peer.tx)?;
                    }
                  }
                }

                info!("{} moved to {:?} @ {:?}", peer.player.name, position, chunk_pos);
              }

              _ => { }
            }
          }
        }
      }
    }

    return Ok(());
  }
}

async fn handle_tcp_connection(server: &Server, mut raw_stream: TcpStream, addr: SocketAddr) {
  info!("TCP connection established: {}", addr);

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
    if server.handle_packet(&msg, addr).is_err() {
      return future::err(std::io::Error::new(ErrorKind::ConnectionRefused, ""));
    };

    return future::ok(());
  });

  let receive_from_others = rx
    .map(|x| Bytes::from(x.into_data()))
    .map(Ok)
    .forward(outgoing);

  pin_mut!(broadcast_incoming, receive_from_others);
  future::select(broadcast_incoming, receive_from_others).await;

  info!("{} disconnected", &addr);
  server.peers.remove(&addr);
}

async fn handle_ws_connection(server: &Server, raw_stream: TcpStream, addr: SocketAddr) {
  let ws_stream = tokio_tungstenite::accept_async(raw_stream).await
    .expect("Error during the websocket handshake occurred");

  info!("WebSocket connection established: {}", addr);

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

  info!("{} disconnected", &addr);
  server.peers.remove(&addr);
}