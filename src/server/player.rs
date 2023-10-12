use tokio_tungstenite::tungstenite::Message;
use futures_channel::mpsc::{unbounded, UnboundedSender};
use glam::{IVec3, ivec3};
use crate::game::player::Player;

pub type Tx = UnboundedSender<Message>;

pub struct ServerPlayer {
  pub tx         : Tx,
  pub player     : Player,
  pub last_chunk : IVec3,
}

impl Default for ServerPlayer {
  fn default() -> Self {
    return Self {
      tx         : unbounded().0,
      player     : Player::default(),
      last_chunk : ivec3(0, 0, 0),
    };
  }
}