//! The main backend interface

use actix::{prelude::*, SystemRunner};
use actix_web::{fs, http, middleware, server, ws, App, Binary};
use capnp::{
    message::{Builder, ReaderOptions},
    serialize_packed::{read_message, write_message},
};
use failure::Error;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use protocol_capnp::{request, response};

/// The server instance
pub struct Server {
    runner: SystemRunner,
}

impl Server {
    /// Create a new server instance
    pub fn new(addr: &str) -> Result<Self, Error> {
        // Build a new actor system
        let system_runner = actix::System::new("ws");

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

        Ok(Server { runner: system_runner })
    }

    /// Start the server
    pub fn start(self) -> i32 {
        self.runner.run()
    }
}

/// The actual websocket
struct WebSocket;

impl Actor for WebSocket {
    type Context = ws::WebsocketContext<Self>;
}

/// Handler for `ws::Message`
impl StreamHandler<ws::Message, ws::ProtocolError> for WebSocket {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        // process websocket messages
        debug!("Message: {:?}", msg);
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => ctx.text(text),
            ws::Message::Binary(bin) => if let Err(e) = self.handle_request(&bin, ctx) {
                error!("Unable to handle request: {}", e);
            },
            ws::Message::Close(reason) => {
                info!("Closing websocket connection: {:?}", reason);
                ctx.stop();
            }
            _ => (),
        }
    }
}

impl WebSocket {
    fn handle_request(&mut self, data: &Binary, ctx: &mut ws::WebsocketContext<Self>) -> Result<(), Error> {
        // Try to read the message
        let reader = read_message(&mut data.as_ref(), ReaderOptions::new())?;
        let request = reader.get_root::<request::Reader>()?;

        // Check the request type
        match request.which() {
            Ok(request::Login(data)) => {
                debug!("User {:?} is trying to Login", data?.get_username());
                let mut message = Builder::new_default();

                {
                    // Set the message data
                    let response = message.init_root::<response::Builder>();
                    let mut login = response.init_login();
                    login.set_success(true);
                }

                // Write the message into a buffer
                let mut data = Vec::new();
                write_message(&mut data, &message)?;

                // Send the response to the websocket
                ctx.binary(data);

                Ok(())
            }
            Ok(request::Logout(())) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
