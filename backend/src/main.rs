extern crate env_logger;
extern crate failure;
extern crate toml;
extern crate webapp;
extern crate webapp_backend;

use failure::Error;
use std::{env::set_var, fs::read_to_string, process::exit};
use webapp::{config::Config, CONFIG_FILENAME};
use webapp_backend::Server;

fn main() -> Result<(), Error> {
    // Parse the configuration
    let config_string = read_to_string(CONFIG_FILENAME)?;
    let config: Config = toml::from_str(&config_string)?;

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
