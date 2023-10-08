use glam::{IVec3, Vec3};
use serde::{Deserialize, Serialize};
use crate::game::world::BlockId;

pub const CHUNK_SIZE: usize = 32;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Chunk {
  pub blocks: Vec<BlockId>,
}

impl Default for Chunk {
  fn default() -> Self {
    return Self {
      blocks: vec![BlockId::AIR; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
    };
  }
}

impl Chunk {
  pub fn get_block(&self, x: usize, y: usize, z: usize) -> BlockId {
    return self.blocks[x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE];
  }

  pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: BlockId) {
    self.blocks[x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE] = block;
  }
}

pub trait ChunkVec3Ext {
  fn to_chunk_pos(&self) -> IVec3;
}

impl ChunkVec3Ext for Vec3 {
  fn to_chunk_pos(&self) -> IVec3 {
    return IVec3::new(
      (self.x / CHUNK_SIZE as f32).floor() as i32,
      (self.y / CHUNK_SIZE as f32).floor() as i32,
      (self.z / CHUNK_SIZE as f32).floor() as i32,
    );
  }
}