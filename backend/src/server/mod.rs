//! Everything related to the actual server implementation

use actix::{prelude::*, SystemRunner};
use actix_web::{
    fs::StaticFiles,
    http::{
        self,
        header::{CONTENT_TYPE, LOCATION},
    },
    middleware::{self, cors::Cors},
    server, App, HttpResponse,
};
use database::DatabaseExecutor;
use diesel::{prelude::*, r2d2::ConnectionManager};
use failure::Error;
use http::{login_credentials, login_session, logout};
use num_cpus;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use r2d2::Pool;
use std::thread;
use webapp::config::Config;

mod tests;

/// The server instance
pub struct Server {
    config: Config,
    runner: SystemRunner,
}

/// Shared mutable application state
pub struct State<T>
where
    T: Actor,
{
    /// The database connection
    pub database: Addr<T>,
}

impl Server {
    /// Create a new server instance
    pub fn new(config: &Config) -> Result<Self, Error> {
        // Build a new actor system
        let runner = actix::System::new("backend");

        // Start database executor actors
        let database_url = format!(
            "postgres://{}:{}@{}/{}",
            config.postgres.username, config.postgres.password, config.postgres.host, config.postgres.database,
        );
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder().build(manager)?;
        let db_addr = SyncArbiter::start(num_cpus::get(), move || DatabaseExecutor(pool.clone()));

        // Create the server
        let config_clone = config.clone();
        let server = server::new(move || {
            App::with_state(State {
                database: db_addr.clone(),
            }).middleware(middleware::Logger::default())
            .configure(|app| {
                Cors::for_app(app)
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_header(CONTENT_TYPE)
                    .max_age(3600)
                    .resource(&config_clone.api.login_credentials, |r| {
                        r.method(http::Method::POST).f(login_credentials)
                    }).resource(&config_clone.api.login_session, |r| {
                        r.method(http::Method::POST).f(login_session)
                    }).resource(&config_clone.api.logout, |r| r.method(http::Method::POST).f(logout))
                    .register()
            }).handler("/", StaticFiles::new(".").unwrap().index_file("index.html"))
        });

        // Create the server url from the given configuration
        let server_url = format!("{}:{}", config.server.ip, config.server.port);

        // Load the SSL Certificate if needed
        if config.server.tls {
            let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
            builder.set_private_key_file(&config.server.key, SslFiletype::PEM)?;
            builder.set_certificate_chain_file(&config.server.cert)?;
            server.bind_ssl(server_url, builder)?.shutdown_timeout(0).start();
        } else {
            server.bind(server_url)?.shutdown_timeout(0).start();
        }

        Ok(Server {
            config: config.clone(),
            runner,
        })
    }

    /// Start the server
    pub fn start(self) -> i32 {
        // Create redirecting server
        if self.config.server.tls {
            let redirect_addr = format!("https://{}:{}", self.config.server.ip, self.config.server.port);
            let bind_addr = format!("http://{}:{}", self.config.server.ip, self.config.server.redirect_port);
            info!(
                "Starting redirect server to redirect from {} to {}",
                bind_addr, redirect_addr
            );
            thread::spawn(move || {
                let sys = actix::System::new("redirect-server");
                if let Ok(s) = server::new(move || {
                    let location = redirect_addr.to_owned();
                    App::new().resource("/", |r| {
                        r.f(move |_| {
                            HttpResponse::PermanentRedirect()
                                .header(LOCATION, location.to_owned())
                                .finish()
                        })
                    })
                }).bind(bind_addr)
                {
                    s.start();
                    sys.run();
                } else {
                    error!("Failed to start redirecting server");
                }
            });
        }
        self.runner.run()
    }
}
