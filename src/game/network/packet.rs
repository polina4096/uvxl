use glam::{IVec3, Vec3};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::game::world::chunk::Chunk;

pub trait Respondable {
  type Response;
}

// server packets
#[repr(u8)]
#[derive(Serialize, Deserialize, Debug)]
pub enum ServerPacket {
  ErrorServerPacket(ErrorServerPacket),
  ClientJoinSuccessServerPacket(ClientJoinSuccessServerPacket),
  PlayerJoinServerPacket(PlayerJoinServerPacket),
  PlayerMoveServerPacket(PlayerMoveServerPacket),
  InitialChunkDataServerPacket(InitialChunkDataServerPacket),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerError {
  PlayerLoggedIn
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitialPlayerData {
  pub uuid     : Uuid,
  pub name     : String,
  pub position : Vec3,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorServerPacket {
  pub error : ServerError,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientJoinSuccessServerPacket {
  pub uuid     : Uuid,
  pub position : Vec3,
  pub players  : Vec<InitialPlayerData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerJoinServerPacket {
  pub uuid    : Uuid,
  pub name    : String,
  pub position: Vec3,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerMoveServerPacket {
  pub uuid     : Uuid,
  pub position : Vec3,
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
  type Response = ClientJoinSuccessServerPacket;
}