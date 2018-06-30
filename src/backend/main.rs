extern crate env_logger;
extern crate failure;
extern crate log;
extern crate webapp;

use failure::Error;
use std::process::exit;
use webapp::Server;

fn main() -> Result<(), Error> {
    // Initialize the logger
    env_logger::init();

    // Create and start the server
    let server = Server::new(option_env!("SERVER_URL").unwrap_or("0.0.0.0:30000"))?;
    exit(server.start());
}
