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
    let server = Server::new(env!("SERVER_URL"))?;
    exit(server.start());
}
