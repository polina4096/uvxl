use uvxl::server::server::Server;

use std::net::SocketAddr;

fn main() {
  pretty_env_logger::init();
  let server = Box::leak(Box::new(Server::new()));
  server.run(SocketAddr::from(([0, 0, 0, 0], 2488))).unwrap();
}
