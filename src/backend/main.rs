extern crate env_logger;
extern crate log;
extern crate webapp;

use webapp::Server;

fn main() {
    // Initialize the logger
    env_logger::init();

    // Start the server
    if let Err(e) = Server::run() {
        println!("Unable to start server: {}", e);
        std::process::exit(1);
    }
}
