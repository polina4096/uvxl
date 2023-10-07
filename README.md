# UVxl [![dependency status](https://deps.rs/repo/github/polina4096/uvxl/status.svg)](https://deps.rs/repo/github/polina4096/uvxl)

Multiplayer voxel sandbox game which runs on Linux, macOS, Windows, and the web using WASM.

## Build Instructions
1. Install the Rust toolchain: https://rustup.rs
2. Clone the repository: `git clone git@github.com:polina4096/uvxl.git`
3. Navigate to repository's directory: `cd uvxl`
4. Build: `cargo build --release`

As of now, the game lacks an integrated server both on native and web. For instructions on how to build and run the server look into [uvxl-server](uvxl-server).

### WASM support
Install [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/), build with: `wasm-pack build --out-dir www/pkg --release`.

## License
Distributed under the MIT license.