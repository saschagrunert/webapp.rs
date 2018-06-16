//! The main backend interface

use actix::prelude::*;
use actix::SystemRunner;
use actix_web::{fs, http, middleware, server, ws, App, Binary};
use failure::Error;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use serde_cbor;
use shared::{LoginResponseData, WsMessage};

use capnp::message::ReaderOptions;
use capnp::serialize_packed::read_message;
use protocol_capnp::request;

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
            ws::Message::Binary(bin) => self.handle_request(&bin, ctx),
            ws::Message::Close(_) => {
                ctx.stop();
            }
            _ => (),
        }
    }
}

impl WebSocket {
    fn handle_request(&mut self, data: &Binary, ctx: &mut ws::WebsocketContext<Self>) {
        let reader = read_message(&mut data.as_ref(), ReaderOptions::new()).unwrap();
        let request = reader.get_root::<request::Reader>().unwrap();

        match request.which() {
            Ok(request::Login(data)) => {
            }
            Ok(request::Logout(())) => {
            }
            Err(capnp::NotInSchema(_)) => {}
        }

        let request: Result<WsMessage, _> = serde_cbor::from_slice(data.as_ref());
        match request {
            Err(e) => error!("Unable to interpret message: {}", e),
            Ok(WsMessage::LoginRequest(d)) => {
                // Check for a login request
                debug!("User {} is trying to auth", d.username);

                // Write the response
                let response_data = WsMessage::LoginResponse(LoginResponseData { success: true });
                match serde_cbor::to_vec(&response_data) {
                    Err(e) => error!("Unable to serialize reponse data: {}", e),
                    Ok(login_response) => ctx.binary(login_response),
                }
            }
            _ => warn!("Unsuppored message type"),
        }
    }
}
