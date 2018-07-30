//! Configruation related structures

#[derive(Clone, Deserialize)]
/// The global configuration
pub struct Config {
    /// The server configuration
    pub server: ServerConfig,

    /// The logger configuration
    pub log: LogConfig,

    /// The database configuration
    pub postgres: PostgresConfig,

    /// The API configuration
    pub api: ApiConfig,
}

#[derive(Clone, Deserialize)]
/// The server configuration
pub struct ServerConfig {
    /// The server IP address
    pub ip: String,

    /// The server port
    pub port: String,

    /// True if the server should use a tls connection
    pub tls: bool,
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

#[derive(Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// The API configuration
pub struct ApiConfig {
    /// The credentials based login API
    pub login_credentials: String,

    /// The session based login API
    pub login_session: String,

    /// The logout API
    pub logout: String,
}
