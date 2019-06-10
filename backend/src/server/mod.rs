//! Everything related to the actual server implementation

use crate::{
    database::DatabaseExecutor,
    http::{login_credentials, login_session, logout},
};
use actix::{prelude::*, SystemRunner};
use actix_cors::Cors;
use actix_files::Files;
use actix_web::{
    http::header::{CONTENT_TYPE, LOCATION},
    middleware,
    web::{get, post, resource},
    App, HttpResponse, HttpServer,
};
use diesel::{prelude::*, r2d2::ConnectionManager};
use failure::Fallible;
use log::{info, warn};
use num_cpus;
use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};
use r2d2::Pool;
use std::thread;
use url::Url;
use webapp::{config::Config, API_URL_LOGIN_CREDENTIALS, API_URL_LOGIN_SESSION, API_URL_LOGOUT};

mod test;

/// The server instance
pub struct Server {
    config: Config,
    runner: SystemRunner,
    url: Url,
}

impl Server {
    /// Create a new server instance
    pub fn from_config(config: &Config) -> Fallible<Self> {
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
        let server = HttpServer::new(move || {
            App::new()
                .data(db_addr.clone())
                .wrap(
                    Cors::new()
                        .allowed_methods(vec!["GET", "POST"])
                        .allowed_header(CONTENT_TYPE)
                        .max_age(3600),
                )
                .wrap(middleware::Logger::default())
                .service(
                    resource(API_URL_LOGIN_CREDENTIALS).route(post().to_async(login_credentials)),
                )
                .service(resource(API_URL_LOGIN_SESSION).route(post().to_async(login_session)))
                .service(resource(API_URL_LOGOUT).route(post().to_async(logout)))
                .service(Files::new("/", "./static/").index_file("index.html"))
        });

        // Create the server url from the given configuration
        let url = Url::parse(&config.server.url)?;

        // Bind the address
        if url.scheme() == "https" {
            server.bind_ssl(&url, Self::build_tls(&config)?)?.start();
        } else {
            server.bind(&url)?.start();
        }

        Ok(Server {
            config: config.to_owned(),
            runner,
            url,
        })
    }

    /// Start the server
    pub fn start(self) -> Fallible<()> {
        // Start the redirecting server
        self.start_redirects();

        // Start the actual main server
        self.runner.run()?;

        Ok(())
    }

    /// Build an SslAcceptorBuilder from a config
    fn build_tls(config: &Config) -> Fallible<SslAcceptorBuilder> {
        let mut tls_builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
        tls_builder.set_private_key_file(&config.server.key, SslFiletype::PEM)?;
        tls_builder.set_certificate_chain_file(&config.server.cert)?;
        Ok(tls_builder)
    }

    fn start_redirects(&self) {
        // Check if we need to create a redirecting server
        if !self.config.server.redirect_from.is_empty() {
            // Prepare needed variables
            let server_url = self.url.clone();
            let urls = self.config.server.redirect_from.to_owned();
            let config_clone = self.config.clone();

            // Create a separate thread for redirecting
            thread::spawn(move || {
                let system = actix::System::new("redirect");
                let url = server_url.clone();

                // Create redirecting server
                let mut server = HttpServer::new(move || {
                    let location = url.clone();
                    App::new().service(resource("/").route(get().to(move || {
                        HttpResponse::PermanentRedirect()
                            .header(LOCATION, location.as_str())
                            .finish()
                    })))
                });

                // Bind the URLs if possible
                for url in &urls {
                    if let Ok(valid_url) = Url::parse(url) {
                        info!(
                            "Starting server to redirect from {} to {}",
                            valid_url, server_url
                        );
                        if valid_url.scheme() == "https" {
                            if let Ok(tls) = Self::build_tls(&config_clone) {
                                server = server.bind_ssl(&valid_url, tls).unwrap();
                            } else {
                                warn!("Unable to build TLS acceptor for server: {}", valid_url);
                            }
                        } else {
                            server = server.bind(&valid_url).unwrap();
                        }
                    } else {
                        warn!("Skipping invalid url: {}", url);
                    }
                }

                // Start the server and the system
                server.start();
                system.run().unwrap();
            });
        }
    }
}
