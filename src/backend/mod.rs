//! The main backend interface

use actix::prelude::*;
use actix::SystemRunner;
use actix_web::{fs, http, middleware, server, ws, App, Binary};
use failure::Error;
use serde_json;
use shared::{LoginResponseData, WsMessage};

/// The server instance
pub struct Server {
    address: String,
    runner: SystemRunner,
}

impl Server {
    /// Create a new server instance
    pub fn new(addr: &str) -> Result<Self, Error> {
        let sys = actix::System::new("ws");

        server::new(|| {
            App::new()
                .middleware(middleware::Logger::default())
                .resource("/ws", |r| r.method(http::Method::GET).f(|r| ws::start(r, WebSocket)))
                .handler("/", fs::StaticFiles::new("static/").index_file("index.html"))
        }).bind(addr)?
            .start();

        Ok(Server {
            runner: sys,
            address: addr.to_owned(),
        })
    }

    /// Start the server
    pub fn start(self) -> i32 {
        info!("Starting server at {}", self.address);
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
            ws::Message::Binary(bin) => self.handle_login_request(&bin, ctx),
            ws::Message::Close(_) => {
                ctx.stop();
            }
            _ => (),
        }
    }
}

impl WebSocket {
    fn handle_login_request(&mut self, data: &Binary, ctx: &mut ws::WebsocketContext<Self>) {
        let request: Result<WsMessage, _> = serde_json::from_slice(data.as_ref());
        match request {
            Err(e) => error!("Unable to interpret message: {}", e),
            Ok(WsMessage::LoginRequest(d)) => {
                // Check for a login request
                debug!("User {} is trying to auth", d.username);

                // Write the response
                let response_data = WsMessage::LoginResponse(LoginResponseData { success: true });
                match serde_json::to_vec(&response_data) {
                    Err(e) => error!("Unable to serialize reponse data: {}", e),
                    Ok(login_response) => ctx.binary(login_response),
                }
            }
            _ => warn!("Unsuppored message type"),
        }
    }
}
