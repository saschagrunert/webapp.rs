extern crate env_logger;
extern crate failure;
extern crate log;
extern crate webapp;

use failure::Error;
use webapp::Server;

fn main() -> Result<(), Error> {
    // Initialize the logger
    env_logger::init();

    // Start the server
    Server::run()?;

    Ok(())
}
