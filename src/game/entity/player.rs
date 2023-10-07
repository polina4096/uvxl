use serde::{Deserialize, Serialize};
use crate::game::entity::{Entity, EntityState};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EntityPlayer {
  state: EntityState,
}

impl Entity for EntityPlayer {
  fn state_mut(&mut self) -> &mut EntityState { &mut self.state }
  fn state(&self) -> &EntityState { &self.state }
}