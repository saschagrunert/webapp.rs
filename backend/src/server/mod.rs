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
use webapp::{config::Config, API_URL_LOGIN_CREDENTIALS, API_URL_LOGIN_SESSION, API_URL_LOGOUT};

mod tests;

/// The server instance
pub struct Server {
    config: Config,
    runner: SystemRunner,
    server_url: String,
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
            config.postgres.username,
            config.postgres.password,
            config.postgres.host,
            config.postgres.database,
        );
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder().build(manager)?;
        let db_addr = SyncArbiter::start(num_cpus::get(), move || DatabaseExecutor(pool.clone()));

        // Create the server
        let server = server::new(move || {
            App::with_state(State {
                database: db_addr.clone(),
            }).middleware(middleware::Logger::default())
            .configure(|app| {
                Cors::for_app(app)
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_header(CONTENT_TYPE)
                    .max_age(3600)
                    .resource(API_URL_LOGIN_CREDENTIALS, |r| {
                        r.method(http::Method::POST).f(login_credentials)
                    }).resource(API_URL_LOGIN_SESSION, |r| {
                        r.method(http::Method::POST).f(login_session)
                    }).resource(API_URL_LOGOUT, |r| r.method(http::Method::POST).f(logout))
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
            server
                .bind_ssl(&server_url, builder)?
                .shutdown_timeout(0)
                .start();
        } else {
            server.bind(&server_url)?.shutdown_timeout(0).start();
        }

        Ok(Server {
            config: config.to_owned(),
            runner,
            server_url,
        })
    }

    /// Start the server
    pub fn start(self) -> i32 {
        // Start the redirecting server
        self.start_redirect();

        // Start the actual main server
        self.runner.run()
    }

    fn start_redirect(&self) {
        // Check if we need to create a redirecting server
        if !self.config.server.redirect_http_from.is_empty() {
            // Prepare needed variables
            let server_url = self.server_url.to_owned();
            let urls = self.config.server.redirect_http_from.to_owned();

            // Create a separate thread for redirecting
            thread::spawn(move || {
                let system = actix::System::new("redirect");
                let url = server_url.to_owned();

                // Create redirecting server
                let mut server = server::new(move || {
                    let location = format!("http://{}", url);
                    App::new().resource("/", |r| {
                        r.f(move |_| {
                            HttpResponse::PermanentRedirect()
                                .header(LOCATION, location.to_owned())
                                .finish()
                        })
                    })
                });

                // Bind the URLs
                for url in &urls {
                    info!("Starting server to redirect from {} to {}", url, server_url);
                    server = server.bind(url).unwrap();
                }

                // Start the server and the system
                server.start();
                system.run();
            });
        }
    }
}
