//! Everything related to the websocket connection

use actix::prelude::*;
use actix_web::{
    ws::{Message, ProtocolError, WebsocketContext},
    Binary,
};
use database::{CreateSession, DeleteSession, UpdateSession};
use failure::Error;
use futures::Future;
use serde_cbor::{from_slice, to_vec};
use server::State;
use token::Token;
use webapp::protocol::{
    request, response, DatabaseError, LoginCredentialsError, LoginSessionError, LogoutError, Request, Response, Session,
};

/// The actual websocket
pub struct WebSocket;

impl Actor for WebSocket {
    type Context = WebsocketContext<Self, State>;
}

/// Handler for `Message`
impl StreamHandler<Message, ProtocolError> for WebSocket {
    fn handle(&mut self, msg: Message, context: &mut Self::Context) {
        match msg {
            Message::Binary(bin) => if let Err(e) = self.handle_request(&bin, context) {
                // If anything in request handling went wrong
                warn!("Unable to send response: {}: Sending generic error", e);
                if let Err(e) = self.send(context, &Response::Error) {
                    // Rare case that it is impossible to send the generic server error
                    warn!("Unable to send generic error: {}: closing connection", e);
                    context.stop();
                }
            },
            Message::Close(reason) => {
                info!("Closing websocket connection: {:?}", reason);
                context.stop();
            }
            e => {
                warn!("Got invalid message: {:?}: closing connection", e);
                context.stop();
            }
        }
    }
}

impl WebSocket {
    pub fn new() -> Self {
        Self {}
    }

    fn handle_request(&mut self, data: &Binary, context: &mut WebsocketContext<Self, State>) -> Result<(), Error> {
        // Try to read the message
        debug!("Got request of len: {}", data.len());
        let request: Request = from_slice(data.as_ref())?;

        // Check the request type
        match request {
            Request::Login(login) => {
                // Check if its a credential or token login type
                match login {
                    request::Login::Credentials {
                        username: u,
                        password: p,
                    } => {
                        let response = Response::Login(response::Login::Credentials(
                            self.handle_request_login_credentials(&u, &p, context),
                        ));

                        // Send the response to the websocket
                        self.send(context, &response)?;
                        Ok(())
                    }
                    request::Login::Session(s) => {
                        let response =
                            Response::Login(response::Login::Session(self.handle_request_login_session(&s, context)));

                        // Send the response to the websocket
                        self.send(context, &response)?;
                        Ok(())
                    }
                }
            }
            Request::Logout(s) => {
                let response = Response::Logout(self.handle_request_logout(s, context));

                // Send the response to the websocket
                self.send(context, &response)?;
                Ok(())
            }
        }
    }

    /// Serialize the data and send it to the websocket
    fn send(&self, context: &mut WebsocketContext<Self, State>, response: &Response) -> Result<(), Error> {
        context.binary(to_vec(response)?);
        Ok(())
    }

    fn handle_request_login_credentials(
        &mut self,
        username: &str,
        password: &str,
        context: &mut WebsocketContext<Self, State>,
    ) -> Result<Session, LoginCredentialsError> {
        debug!("User {} is trying to login", username);

        // Error if username and password are invalid
        if username.is_empty() || password.is_empty() || username != password {
            debug!("Wrong username or password");
            return Err(LoginCredentialsError::WrongUsernamePassword);
        }

        // Create a new session
        let session = context
            .state()
            .database
            .send(CreateSession(Token::create(username)?))
            .wait()
            .map_err(|_| DatabaseError::Communication)??;

        // Return the session
        Ok(session)
    }

    fn handle_request_login_session(
        &mut self,
        session: &Session,
        context: &mut WebsocketContext<Self, State>,
    ) -> Result<Session, LoginSessionError> {
        debug!("Session token {} wants to be renewed", session.token);

        // Try to verify and create a new session
        let new_session = context
            .state()
            .database
            .send(UpdateSession {
                old_token: session.token.to_owned(),
                new_token: Token::verify(&session.token)?,
            })
            .wait()??;

        // Return the new session
        Ok(new_session)
    }

    fn handle_request_logout(
        &mut self,
        session: Session,
        context: &mut WebsocketContext<Self, State>,
    ) -> Result<(), LogoutError> {
        // Remove the session from the internal storage
        context.state().database.send(DeleteSession(session.token)).wait()??;

        Ok(())
    }
}
