use glam::{IVec3, Vec3};
use serde::{Serialize, Deserialize};
use crate::game::world::chunk::Chunk;

pub trait Respondable {
  type Response;
}

// server packets
#[repr(u8)]
#[derive(Serialize, Deserialize, Debug)]
pub enum ServerPacket {
  ClientJoinServerPacket(ClientJoinServerPacket),
  PlayerJoinServerPacket(PlayerJoinServerPacket),
  PlayerMoveServerPacket(PlayerMoveServerPacket),
  InitialChunkDataServerPacket(InitialChunkDataServerPacket),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientJoinServerPacket {
  pub success : bool,
  pub reason  : Option<String>,
  pub players : Vec<(String, Vec3)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerJoinServerPacket {
  pub name: String,
  pub position: Vec3,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerMoveServerPacket {
  pub name: String,
  pub position: Vec3,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InitialChunkDataServerPacket {
  pub chunk    : Chunk,
  pub position : IVec3,
}

// client packets
#[repr(u8)]
#[derive(Serialize, Deserialize, Debug)]
pub enum ClientPacket {
  ClientJoinClientPacket(ClientJoinClientPacket),
  ClientMovePacket(ClientMovePacket),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientJoinClientPacket {
  pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientMovePacket {
  pub position: Vec3,
}

impl Respondable for ClientJoinClientPacket {
  type Response = ClientJoinServerPacket;
}