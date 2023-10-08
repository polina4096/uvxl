use crate::server::world::chunk_manager::ServerChunkManager;

pub struct ServerWorld {
  pub chunk_manager: ServerChunkManager,
}

impl Default for ServerWorld {
  fn default() -> Self {
    return Self {
      chunk_manager: ServerChunkManager::default(),
    };
  }
}