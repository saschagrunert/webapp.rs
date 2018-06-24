//! Everything related to the websocket connection

use actix::prelude::*;
use actix_web::{
    ws::{Message, ProtocolError, WebsocketContext},
    Binary,
};
use backend::server::{ServerError, State};
use capnp::{
    message::{Builder, ReaderOptions},
    serialize_packed::{read_message, write_message},
};
use failure::Error;
use protocol_capnp::{request, response};

/// The actual websocket
pub struct WebSocket;

impl Actor for WebSocket {
    type Context = WebsocketContext<Self, State>;
}

/// Handler for `Message`
impl StreamHandler<Message, ProtocolError> for WebSocket {
    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) {
        match msg {
            Message::Binary(bin) => if let Err(e) = self.handle_request(&bin, ctx) {
                warn!("Unable to send succeeding response: {}", e);
                // Try to send the error response
                match self.create_error_response(&e.to_string()) {
                    Ok(d) => ctx.binary(d),
                    Err(e) => error!("Unable to send error: {}", e),
                }
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
    fn handle_request(&mut self, data: &Binary, ctx: &mut WebsocketContext<Self, State>) -> Result<(), Error> {
        // Try to read the message
        let reader = read_message(&mut data.as_ref(), ReaderOptions::new())?;
        let request = reader.get_root::<request::Reader>()?;

        // Create a new message builder
        let mut message = Builder::new_default();
        let mut response_data = Vec::new();

        // Check the request type
        match request.which() {
            Ok(request::Login(data)) => {
                // Check if its a credential or token login type
                match data?.which() {
                    Ok(request::login::Credentials(d)) => {
                        let v = d?;
                        let username = v.get_username()?;
                        let password = v.get_password()?;
                        debug!("User {} is trying to login", username);

                        // Error if username and password are invalid
                        if username.is_empty() || password.is_empty() {
                            debug!("Wrong username or password");
                            return Err(ServerError::WrongUsernamePassword.into());
                        }

                        // Create a new token
                        let token = ctx.state().store.create(username)?;

                        // Create the response
                        message.init_root::<response::Builder>().init_login().set_token(&token);

                        // Write the message into a buffer
                        write_message(&mut response_data, &message)?;

                        // Send the response to the websocket
                        ctx.binary(response_data);
                        Ok(())
                    }
                    Ok(request::login::Token(token_data)) => {
                        let token = token_data?;
                        debug!("Token {} wants to be renewed", token);

                        // Try to verify and create a new token
                        let new_token = ctx.state().store.verify(token)?;

                        // Create the response
                        message
                            .init_root::<response::Builder>()
                            .init_login()
                            .set_token(&new_token);

                        // Write the message into a buffer
                        write_message(&mut response_data, &message)?;

                        // Send the response to the websocket
                        ctx.binary(response_data);
                        Ok(())
                    }
                    Err(e) => Err(e.into()),
                }
            }
            Ok(request::Logout(token)) => {
                ctx.state().store.remove(token?)?;
                message.init_root::<response::Builder>().set_logout(());
                write_message(&mut response_data, &message)?;
                ctx.binary(response_data);
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Create an error response from a given description string
    fn create_error_response(&self, description: &str) -> Result<Vec<u8>, Error> {
        let mut message = Builder::new_default();

        // Create the message
        message
            .init_root::<response::Builder>()
            .init_error()
            .set_description(description);

        // Write the message into a buffer
        let mut data = Vec::new();
        write_message(&mut data, &message)?;

        Ok(data)
    }
}
