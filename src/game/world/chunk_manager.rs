use std::collections::HashMap;
use glam::IVec3;
use crate::game::world::chunk::Chunk;

pub struct ChunkManager {
  pub chunks: HashMap<IVec3, Chunk>,
}

impl Default for ChunkManager {
  fn default() -> Self {
    return Self {
      chunks: Default::default(),
    };
  }
}