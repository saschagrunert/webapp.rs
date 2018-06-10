#[macro_use]
extern crate log;
extern crate env_logger;
extern crate webapp;

use webapp::Server;

fn main() {
    // Initialize the logger
    env_logger::init();

    // Start the server
    if let Err(e) = Server::run() {
        error!("Unable to start server: {}", e);
    }
}
