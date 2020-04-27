//! Configuration related structures
use anyhow::Result;
use serde::Deserialize;
use std::{fs::read_to_string, path::PathBuf};

#[derive(Clone, Deserialize)]
/// The global configuration
pub struct Config {
    /// The server configuration
    pub server: ServerConfig,

    /// The logger configuration
    pub log: LogConfig,

    /// The database configuration
    pub postgres: PostgresConfig,
}

impl Config {
    /// Creates a new `Config` instance using the parameters found in the given
    /// TOML configuration file. If the file could not be found or the file is
    /// invalid, an `Error` will be returned.
    pub fn from_file(filename: &str) -> Result<Self> {
        Ok(toml::from_str(&read_to_string(filename)?)?)
    }
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// The server configuration
pub struct ServerConfig {
    /// The full server URL
    pub url: String,

    /// The server certificate
    pub cert: PathBuf,

    /// The server key
    pub key: PathBuf,

    /// The redirecting URLs
    pub redirect_from: Vec<String>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// The logger configuration
pub struct LogConfig {
    /// The logging level of actix-web
    pub actix_web: String,

    /// The logging level of the application
    pub webapp: String,
}

#[derive(Clone, Deserialize)]
/// The database configuration
pub struct PostgresConfig {
    /// The full host to the database
    pub host: String,

    /// The username for the database
    pub username: String,

    /// The password for the database
    pub password: String,

    /// The database to be used
    pub database: String,
}
