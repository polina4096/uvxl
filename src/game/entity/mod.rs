use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

pub mod player;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EntityState {
  pub title: Option<String>,
  pub position: Vec3,
  pub velocity: Vec3,
  pub rotation: Quat,
}

pub trait Entity {
  fn state_mut(&mut self) -> &mut EntityState;
  fn state(&self) -> &EntityState;
}