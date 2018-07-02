extern crate env_logger;
extern crate failure;
extern crate log;
extern crate toml;
extern crate webapp;

#[macro_use]
extern crate serde_derive;

use failure::Error;
use std::{env::set_var, fs::read_to_string, process::exit};
use webapp::Server;

#[derive(Deserialize)]
struct Config {
    server: ServerConfig,
    log: LogConfig,
}

#[derive(Deserialize)]
struct ServerConfig {
    ip: String,
    port: String,
}

#[derive(Deserialize)]
struct LogConfig {
    actix_web: String,
    webapp: String,
}

fn main() -> Result<(), Error> {
    // Parse the configuration
    let config_string = read_to_string("Config.toml")?;
    let config: Config = toml::from_str(&config_string)?;

    // Set the logging verbosity
    set_var(
        "RUST_LOG",
        format!("actix_web={},webapp={}", config.log.actix_web, config.log.webapp),
    );

    // Initialize the logger
    env_logger::init();

    // Create and start the server
    let server_url = format!("{}:{}", config.server.ip, config.server.port);
    let server = Server::new(&server_url)?;

    // Start the server
    exit(server.start());
}
