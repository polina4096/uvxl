use uuid::Uuid;
use crate::game::entity::player::EntityPlayer;

#[derive(Debug)]
pub struct Player {
  pub uuid: Uuid,
  pub name: String,
  pub entity: EntityPlayer,
}

impl Default for Player {
  fn default() -> Self {
    Self {
      name: String::new(),
      uuid: Uuid::nil(),
      entity: Default::default(),
    }
  }
}