//! Configuration related structures

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

#[derive(Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// The server configuration
pub struct ServerConfig {
    /// The full server URL
    pub url: String,

    /// Use letsencrypt instead of own certificates
    pub use_acme: bool,

    /// The server certificate
    pub cert: String,

    /// The server key
    pub key: String,

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
