#[derive(Clone, Copy)]
pub enum Side {
  Top    , // Y+
  Bottom , // Y-

  Right  , // Z+
  Left   , // Z-

  Front  , // X+
  Back   , // X-
}