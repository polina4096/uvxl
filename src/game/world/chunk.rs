use glam::{IVec3, Vec3};
use serde::{Deserialize, Serialize};
use crate::game::world::BlockId;

// #[macro_export]
// macro_rules! serde_array { ($m:ident, $n:expr) => {
//   pub mod $m {
//     use std::{ptr, mem, panic};
//     use serde::{Deserialize, Deserializer, de};
//     pub fn serialize<S, T>(array: &[T], serializer: S) -> Result<S::Ok, S::Error>
//       where S: Serializer, T: Serialize {
//       array.serialize(serializer)
//     }
//
//     use super::*;
//
//     pub fn deserialize<'de, D, T>(deserializer: D) -> Result<[T; $n], D::Error>
//     where D: Deserializer<'de>, T: Deserialize<'de> + 'de {
//       let slice: Vec<T> = Deserialize::deserialize(deserializer)?;
//       if slice.len() != $n {
//         return Err(de::Error::custom("input slice has wrong length"));
//       }
//       unsafe {
//         let mut result: [T; $n] = mem::zeroed();
//         for (src, dst) in slice.into_iter().zip(&mut result[..]) {
//           ptr::write(dst, src);
//         }
//         Ok(result)
//       }
//     }
//   }
// }}

// serde_array!(a32768, 32768);

pub const CHUNK_SIZE: usize = 32;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Chunk {
  // #[serde(with = "a32768")]
  pub blocks: Vec<BlockId>,//[BlockId; 32 * 32 * 32],
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