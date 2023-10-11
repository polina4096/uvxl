use crate::game::entity::player::EntityPlayer;

#[derive(Debug)]
pub struct Player {
  pub name: String,
  pub entity: EntityPlayer,
}

impl Default for Player {
  fn default() -> Self {
    Self {
      name: String::new(),
      entity: Default::default(),
    }
  }
}