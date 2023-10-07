use anyhow::Result;
use winit::event_loop::EventLoopProxy;
use std::net::SocketAddr;

use crate::app::UVxlEvent;

cfg_if::cfg_if! {
  if #[cfg(target_arch = "wasm32")] {
    mod wasm;
    mod imp { pub use super::wasm::*; }
  } else {
    mod native;
    mod imp { pub use super::native::*; }
  }
}

pub struct Connection(imp::Connection);

impl Connection {
  pub fn new(address: SocketAddr, callback: EventLoopProxy<UVxlEvent>) -> Result<Self> { imp::Connection::new(address, callback).map(Self) }

  pub fn send(&mut self, packet: impl serde::Serialize) -> Result<()> { imp::Connection::send(&mut self.0, packet) }
}

