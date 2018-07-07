//! Everything related to the actual server implementation

use actix::{prelude::*, SystemRunner};
use actix_web::{fs, http, middleware, server, ws, App};
use backend::{database::executor::DatabaseExecutor, websocket::WebSocket};
use config::Config;
use diesel::{prelude::*, r2d2::ConnectionManager};
use failure::Error;
use num_cpus;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use r2d2::Pool;

#[derive(Debug, Fail)]
pub enum ServerError {
    #[fail(display = "wrong username or password")]
    WrongUsernamePassword,

    #[fail(display = "unable to create token")]
    CreateToken,

    #[fail(display = "unable to verify token")]
    VerifyToken,

    #[fail(display = "unable to insert token into database")]
    InsertToken,

    #[fail(display = "unable to update token within database")]
    UpdateToken,
}

/// The server instance
pub struct Server {
    runner: SystemRunner,
}

/// Shared mutable application state
pub struct State {
    // The database connection
    pub database: Addr<Syn, DatabaseExecutor>,
}

impl Server {
    /// Create a new server instance
    pub fn new(config: &Config) -> Result<Self, Error> {
        // Build a new actor system
        let runner = actix::System::new("ws");

        // Start database executor actors
        let database_url = format!(
            "postgres://{}:{}@{}/{}",
            config.postgres.username, config.postgres.password, config.postgres.host, config.postgres.database,
        );
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder().build(manager)?;
        let db_addr = SyncArbiter::start(num_cpus::get(), move || DatabaseExecutor(pool.clone()));

        // Create the server
        let server = server::new(move || {
            App::with_state(State {
                database: db_addr.clone(),
            }).middleware(middleware::Logger::default())
                .resource("/ws", |r| {
                    r.method(http::Method::GET).f(|r| ws::start(r, WebSocket::new()))
                })
                .handler("/", fs::StaticFiles::new("static").index_file("index.html"))
        });

        // Create the server url from the given configuration
        let server_url = format!("{}:{}", config.server.ip, config.server.port);

        // Load the SSL Certificate if needed
        if config.server.tls {
            let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
            builder.set_private_key_file("tls/key.pem", SslFiletype::PEM)?;
            builder.set_certificate_chain_file("tls/crt.pem")?;
            server.bind_ssl(server_url, builder)?.shutdown_timeout(0).start();
        } else {
            server.bind(server_url)?.shutdown_timeout(0).start();
        }

        Ok(Server { runner })
    }

    /// Start the server
    pub fn start(self) -> i32 {
        self.runner.run()
    }
}

#[cfg(test)]
mod tests {
    extern crate toml;

    use super::*;
    use std::fs::read_to_string;
    use CONFIG_FILENAME;

    #[test]
    fn succeed_to_create_a_server() {
        let config: Config = toml::from_str(&read_to_string(CONFIG_FILENAME).unwrap()).unwrap();
        assert!(Server::new(&config).is_ok());
    }

    #[test]
    fn fail_to_create_a_server_with_wrong_addr() {
        let mut config: Config = toml::from_str(&read_to_string(CONFIG_FILENAME).unwrap()).unwrap();
        config.server.ip = "".to_owned();
        assert!(Server::new(&config).is_err());
    }

    #[test]
    fn fail_to_create_a_server_with_wrong_port() {
        let mut config: Config = toml::from_str(&read_to_string(CONFIG_FILENAME).unwrap()).unwrap();
        config.server.port = "10".to_owned();
        assert!(Server::new(&config).is_err());
    }
}
