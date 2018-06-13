extern crate env_logger;
extern crate failure;
extern crate log;
extern crate webapp;

use failure::Error;
use std::process::exit;
use webapp::Server;

fn main() -> Result<(), Error> {
    // Initialize the logger
    std::env::set_var("RUST_LOG", "actix_web=info,webapp=trace");
    env_logger::init();

    // Create and start the server
    let server = Server::new("0.0.0.0:30000")?;
    exit(server.start());
}
