use std::sync::atomic::AtomicUsize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerSettings {
  pub vertical_render_distance   : AtomicUsize,
  pub horizontal_render_distance : AtomicUsize,
}