//! Everything related to the websocket connection

use actix::prelude::*;
use actix_web::{
    ws::{Message, ProtocolError, WebsocketContext},
    Binary,
};
use backend::{
    database::executor::{CreateSession, DeleteSession, UpdateSession},
    server::{ServerError, State},
    token::Token,
};
use capnp::{
    self,
    message::{Builder, HeapAllocator, ReaderOptions},
    serialize_packed::{read_message, write_message},
    text,
};
use failure::Error;
use futures::Future;
use protocol_capnp::{request, response};

/// The actual websocket
pub struct WebSocket {
    builder: Builder<HeapAllocator>,
    data: Vec<u8>,
}

impl Actor for WebSocket {
    type Context = WebsocketContext<Self, State>;
}

/// Handler for `Message`
impl StreamHandler<Message, ProtocolError> for WebSocket {
    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) {
        match msg {
            Message::Binary(bin) => if let Err(e) = self.handle_request(&bin, ctx) {
                warn!("Unable to send response: {}", e);
            },
            Message::Close(reason) => {
                info!("Closing websocket connection: {:?}", reason);
                ctx.stop();
            }
            e => warn!("Got invalid message: {:?}", e),
        }
    }
}

impl WebSocket {
    pub fn new() -> Self {
        Self {
            builder: Builder::new_default(),
            data: Vec::new(),
        }
    }

    fn handle_request(&mut self, data: &Binary, ctx: &mut WebsocketContext<Self, State>) -> Result<(), Error> {
        // Try to read the message
        let reader = read_message(&mut data.as_ref(), ReaderOptions::new())?;
        let request = reader.get_root::<request::Reader>()?;

        // Check the request type
        match request.which() {
            Ok(request::Login(data)) => {
                // Check if its a credential or token login type
                match data.which() {
                    Ok(request::login::Credentials(d)) => {
                        // Create an error response if needed
                        if let Err(e) = self.handle_request_login_credentials(d, ctx) {
                            self.builder
                                .init_root::<response::Builder>()
                                .init_login()
                                .set_error(&e.to_string());
                            self.write()?;
                        }

                        // Send the response to the websocket
                        self.send(ctx);
                        Ok(())
                    }
                    Ok(request::login::Token(d)) => {
                        // Create an error response if needed
                        if let Err(e) = self.handle_request_login_token(d, ctx) {
                            self.builder
                                .init_root::<response::Builder>()
                                .init_login()
                                .set_error(&e.to_string());
                            self.write()?;
                        }

                        // Send the response to the websocket
                        self.send(ctx);
                        Ok(())
                    }
                    Err(e) => Err(e.into()),
                }
            }
            Ok(request::Logout(d)) => {
                if let Err(e) = self.handle_request_logout(d, ctx) {
                    self.builder
                        .init_root::<response::Builder>()
                        .init_logout()
                        .set_error(&e.to_string());
                    self.write()?;
                }

                // Send the response to the websocket
                self.send(ctx);
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }

    fn write(&mut self) -> Result<&[u8], Error> {
        // Clear the data before serialization
        self.data.clear();

        // Serialize and return
        write_message(&mut self.data, &self.builder)?;
        Ok(&self.data)
    }

    fn send(&self, ctx: &mut WebsocketContext<Self, State>) {
        ctx.binary(self.data.clone());
    }

    fn handle_request_login_credentials(
        &mut self,
        data: request::login::credentials::Reader,
        ctx: &mut WebsocketContext<Self, State>,
    ) -> Result<&[u8], Error> {
        let username = data.get_username()?;
        let password = data.get_password()?;
        debug!("User {} is trying to login", username);

        // Error if username and password are invalid
        if username.is_empty() || password.is_empty() || username != password {
            debug!("Wrong username or password");
            return Err(ServerError::WrongUsernamePassword.into());
        }

        // Create a new token
        let token = ctx
            .state()
            .database
            .send(CreateSession {
                id: Token::create(username)?,
            })
            .wait()??;

        // Create the response
        self.builder
            .init_root::<response::Builder>()
            .init_login()
            .set_token(&token.id);

        // Write the message into a buffer
        self.write()
    }

    fn handle_request_login_token(
        &mut self,
        data: Result<text::Reader, capnp::Error>,
        ctx: &mut WebsocketContext<Self, State>,
    ) -> Result<&[u8], Error> {
        // Read the data
        let token = data?;
        debug!("Token {} wants to be renewed", token);

        // Try to verify and create a new token
        let new_token = ctx
            .state()
            .database
            .send(UpdateSession {
                old_id: token.to_owned(),
                new_id: Token::verify(token)?,
            })
            .wait()??;

        // Create the response
        self.builder
            .init_root::<response::Builder>()
            .init_login()
            .set_token(&new_token.id);

        // Write the message into a buffer
        self.write()
    }

    fn handle_request_logout(
        &mut self,
        data: Result<text::Reader, capnp::Error>,
        ctx: &mut WebsocketContext<Self, State>,
    ) -> Result<&[u8], Error> {
        // Remove the token from the internal storage
        ctx.state()
            .database
            .send(DeleteSession { id: data?.to_owned() })
            .wait()??;

        // Create the response
        self.builder
            .init_root::<response::Builder>()
            .init_logout()
            .set_success(());

        // Write the message into a buffer
        self.write()
    }
}
