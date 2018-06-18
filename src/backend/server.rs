//! Everything related to the actual server implementation

use actix::{prelude::*, SystemRunner};
use actix_web::{fs, http, middleware, server, ws, App};
use backend::websocket::WebSocket;
use failure::Error;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

#[derive(Debug, Fail)]
pub enum ServerError {
    #[fail(display = "unimplemented request message")]
    UnimplementedRequest,

    #[fail(display = "wrong username or password")]
    WrongUsernamePassword,

    #[fail(display = "unable to create token")]
    CreateToken,

    #[fail(display = "unable to verify token")]
    VerifyToken,
}

/// The server instance
pub struct Server {
    runner: SystemRunner,
}

impl Server {
    /// Create a new server instance
    pub fn new(addr: &str) -> Result<Self, Error> {
        // Build a new actor system
        let runner = actix::System::new("ws");

        // Load the SSL Certificate
        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
        builder.set_private_key_file("tls/key.pem", SslFiletype::PEM)?;
        builder.set_certificate_chain_file("tls/crt.pem")?;

        // Create the server
        server::new(|| {
            App::new()
                .middleware(middleware::Logger::default())
                .resource("/ws", |r| r.method(http::Method::GET).f(|r| ws::start(r, WebSocket)))
                .handler("/", fs::StaticFiles::new("static/").index_file("index.html"))
        }).bind_ssl(addr, builder)?
            .shutdown_timeout(0)
            .start();

        Ok(Server { runner })
    }

    /// Start the server
    pub fn start(self) -> i32 {
        self.runner.run()
    }
}
