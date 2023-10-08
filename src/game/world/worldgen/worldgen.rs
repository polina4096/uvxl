use glam::IVec3;
use crate::game::world::BlockId;
use crate::game::world::chunk::{Chunk, CHUNK_SIZE};

pub struct WorldGen {

}

impl WorldGen {
  pub fn generate(&self, chunk_pos: IVec3) -> Chunk {
    let mut chunk = Chunk::default();

    for x in 0 .. CHUNK_SIZE {
      for z in 0 .. CHUNK_SIZE {
        let height
          = ((chunk_pos.x as usize * CHUNK_SIZE + x) as f32 * 0.1).sin()
          * ((chunk_pos.x as usize * CHUNK_SIZE + z) as f32 * 0.1).cos()
          * 10.0 + 10.0;

        for y in 0 .. (height.round() as usize).max(CHUNK_SIZE) {
          let absolute_y = chunk_pos.y as usize * CHUNK_SIZE + y;
          if absolute_y < 32 {
            chunk.set_block(x, y, z, BlockId::TEST);
          }

        }

      }
    }

    return chunk;
  }
}