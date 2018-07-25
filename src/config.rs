//! Configruation related structures

#[derive(Deserialize)]
/// The global configuration
pub struct Config {
    /// The server configuration
    pub server: ServerConfig,

    /// The logger configuration
    pub log: LogConfig,

    /// The database configuration
    pub postgres: PostgresConfig,
}

#[derive(Deserialize)]
/// The server configuration
pub struct ServerConfig {
    /// The server IP address
    pub ip: String,

    /// The server port
    pub port: String,

    /// True if the server should use a tls connection
    pub tls: bool,
}

#[derive(Deserialize)]
/// The logger configuration
pub struct LogConfig {
    /// The logging level of actix-web
    pub actix_web: String,

    /// The logging level of the application
    pub webapp: String,
}

#[derive(Deserialize)]
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
