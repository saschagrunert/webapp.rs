//! The main backend interface

use actix::{prelude::*, SystemRunner};
use actix_web::{fs, http, middleware, server, ws, App, Binary};
use capnp::{
    message::{Builder, ReaderOptions},
    serialize_packed::{read_message, write_message},
};
use failure::Error;
use jsonwebtoken::{decode, encode, Header, Validation};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use protocol_capnp::{request, response};
use time::get_time;

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

#[derive(Debug, Deserialize, Serialize)]
struct Claim {
    sub: String,
    exp: i64,
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

/// The actual websocket
struct WebSocket;

impl Actor for WebSocket {
    type Context = ws::WebsocketContext<Self>;
}

/// Handler for `ws::Message`
impl StreamHandler<ws::Message, ws::ProtocolError> for WebSocket {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        // process websocket messages
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => ctx.text(text),
            ws::Message::Binary(bin) => if let Err(e) = self.handle_request(&bin, ctx) {
                warn!("Unable to send succeeding response: {}", e);
                // Try to send the error response
                match self.create_error_response(&e.to_string()) {
                    Ok(d) => ctx.binary(d),
                    Err(e) => error!("Unable to send error: {}", e),
                }
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
                // Create a new message builder
                let mut message = Builder::new_default();
                let mut response_data = Vec::new();

                // Check if its a credential or token login type
                match data?.which() {
                    Ok(request::login::Credentials(d)) => {
                        let v = d?;
                        let username = v.get_username()?;
                        let password = v.get_password()?;
                        debug!("User {} is trying to login", username);

                        // For now, error if username and password does not match
                        if username != password {
                            debug!("Username and password does not match");
                            return Err(ServerError::WrongUsernamePassword.into());
                        }

                        // Else create a "secret" token for the response
                        {
                            let response = message.init_root::<response::Builder>();
                            let mut login = response.init_login();

                            let token = self.create_token(username, 3600)?;
                            debug!("Token: {}", token);
                            login.set_token(&token);
                        }

                        // Write the message into a buffer
                        write_message(&mut response_data, &message)?;

                        // Send the response to the websocket
                        ctx.binary(response_data);
                        Ok(())
                    }
                    Ok(request::login::Token(token_data)) => {
                        let token = token_data?;
                        debug!("Token {} wants to be renewed", token);

                        {
                            // Try to verify and create a new token
                            let new_token = self.verify_token(token)?;
                            debug!("New token: {}", new_token);

                            // Create the response
                            let response = message.init_root::<response::Builder>();
                            let mut login = response.init_login();
                            login.set_token(&new_token);
                        }

                        // Write the message into a buffer
                        write_message(&mut response_data, &message)?;

                        // Send the response to the websocket
                        ctx.binary(response_data);
                        Ok(())
                    }
                    Err(e) => Err(e.into()),
                }
            }
            Err(e) => Err(e.into()),
            _ => Err(ServerError::UnimplementedRequest.into()),
        }
    }

    /// Create a new default token for a given username and a validity in seconds
    fn create_token(&self, username: &str, validity: i64) -> Result<String, Error> {
        let claim = Claim {
            sub: username.to_owned(),
            exp: get_time().sec + validity,
        };
        encode(&Header::default(), &claim, b"secret").map_err(|_| ServerError::CreateToken.into())
    }

    /// Verify the validity of a token and get a new one
    fn verify_token(&self, token: &str) -> Result<String, Error> {
        let data = decode::<Claim>(token, b"secret", &Validation::default())
            .map_err(|_| Error::from(ServerError::VerifyToken))?;
        self.create_token(&data.claims.sub, 3600)
    }

    /// Create an error response from a given description string
    fn create_error_response(&self, description: &str) -> Result<Vec<u8>, Error> {
        let mut message = Builder::new_default();

        {
            let response = message.init_root::<response::Builder>();
            let mut error = response.init_error();
            error.set_description(description);
        }

        // Write the message into a buffer
        let mut data = Vec::new();
        write_message(&mut data, &message)?;

        Ok(data)
    }
}
