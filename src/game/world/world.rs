use crate::game::player::Player;
use crate::game::world::chunk_manager::ChunkManager;

pub struct World {
  pub chunk_manager: ChunkManager,
  pub players: Vec<Player>,
}

impl Default for World {
  fn default() -> Self {
    return Self {
      chunk_manager: ChunkManager::default(),
      players: vec![],
    };
  }
}