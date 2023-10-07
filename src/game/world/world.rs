use crate::game::world::chunk_manager::ChunkManager;

pub struct World {
  pub chunk_manager: ChunkManager,
}

impl Default for World {
  fn default() -> Self {
    return Self {
      chunk_manager: ChunkManager::default(),
    };
  }
}