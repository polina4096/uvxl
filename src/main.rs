#[cfg(not(target_arch = "wasm32"))]
use pollster::FutureExt;

fn main() {
  #[cfg(not(target_arch = "wasm32"))]
  uvxl::run()
    .block_on();
}
