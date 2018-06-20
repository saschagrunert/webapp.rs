//! Everything related to the websocket connection

use actix::prelude::*;
use actix_web::{
    ws::{Message, ProtocolError, WebsocketContext},
    Binary,
};
use backend::{
    server::{ServerError, State},
    token::Token,
};
use capnp::{
    message::{Builder, ReaderOptions},
    serialize_packed::{read_message, write_message},
};
use failure::Error;
use protocol_capnp::{request, response};
use std::sync::Arc;

/// The actual websocket
pub struct WebSocket;

impl Actor for WebSocket {
    type Context = WebsocketContext<Self, Arc<State>>;
}

/// Handler for `Message`
impl StreamHandler<Message, ProtocolError> for WebSocket {
    fn handle(&mut self, msg: Result<Option<Message>, ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Err(e) => error!("Error handling message: {}", e),
            Ok(data) => if let Some(m) = data {
                // process websocket messages
                match m {
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
            } else {
                info!("No valid message data found");
            },
        }
    }
}

impl WebSocket {
    fn handle_request(&mut self, data: &Binary, ctx: &mut WebsocketContext<Self, Arc<State>>) -> Result<(), Error> {
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

                            let token = Token::create(username, 3600)?;
                            debug!("Token: {}", token);
                            login.set_token(&token);

                            // Save the token for validation
                            // TODO: check if token is already present
                            ctx.state()
                                .tokens
                                .try_borrow_mut()?
                                .insert(username.to_owned(), token.to_owned());
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
                            let new_token = Token::verify(token)?;
                            debug!("New token: {}", new_token);

                            // Create the response
                            let response = message.init_root::<response::Builder>();
                            let mut login = response.init_login();
                            login.set_token(&new_token);

                            // TODO: update the token within the application state
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
