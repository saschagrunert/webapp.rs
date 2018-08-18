extern crate env_logger;
extern crate failure;
extern crate webapp;
extern crate webapp_backend;

use failure::Fallible;
use std::{env::set_var, process::exit};
use webapp::{config::Config, CONFIG_FILENAME};
use webapp_backend::Server;

fn main() -> Fallible<()> {
    // Parse the configuration
    let config = Config::new(CONFIG_FILENAME)?;

    // Set the logging verbosity
    set_var(
        "RUST_LOG",
        format!(
            "actix_web={},webapp={}",
            config.log.actix_web, config.log.webapp
        ),
    );

    // Initialize the logger
    env_logger::init();

    // Create and start the server
    let server = Server::new(&config)?;

    // Start the server
    exit(server.start());
}
