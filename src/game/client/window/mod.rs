pub mod server_join;

use crate::app::App;

pub type WindowStack = Vec<Box<dyn Window>>;

pub trait Window {
  fn draw(&mut self, app: &mut App);
  fn id(&self) -> WindowId;
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum WindowId {
  ServerJoin,
  PacketDebugger,
}