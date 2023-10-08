use serde::{Deserialize, Serialize};

pub mod world;
pub mod chunk;
pub mod chunk_manager;
pub mod worldgen;

#[derive(Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Serialize, Deserialize)]
pub struct BlockId(u16);

impl BlockId {
  pub const AIR   : BlockId = BlockId(0);
  pub const TEST  : BlockId = BlockId(1);
  pub const PANEL : BlockId = BlockId(2);
}