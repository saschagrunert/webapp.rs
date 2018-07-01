//! Everything related to the websocket connection

use actix::prelude::*;
use actix_web::{
    ws::{Message, ProtocolError, WebsocketContext},
    Binary,
};
use backend::server::{ServerError, State};
use capnp::{
    self,
    message::{Builder, HeapAllocator, ReaderOptions},
    serialize_packed::{read_message, write_message},
    text,
};
use failure::Error;
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
        let token = ctx.state().store.insert(username)?;

        // Create the response
        self.builder
            .init_root::<response::Builder>()
            .init_login()
            .set_token(&token);

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
        let new_token = ctx.state().store.verify(token)?;

        // Create the response
        self.builder
            .init_root::<response::Builder>()
            .init_login()
            .set_token(&new_token);

        // Write the message into a buffer
        self.write()
    }

    fn handle_request_logout(
        &mut self,
        data: Result<text::Reader, capnp::Error>,
        ctx: &mut WebsocketContext<Self, State>,
    ) -> Result<&[u8], Error> {
        // Remove the token from the internal storage
        ctx.state().store.remove(data?)?;

        // Create the response
        self.builder
            .init_root::<response::Builder>()
            .init_logout()
            .set_success(());

        // Write the message into a buffer
        self.write()
    }
}

#[cfg(test)]
mod tests {
    extern crate futures;

    use self::futures::Stream;
    use super::*;
    use actix_web::{test::TestServer, ws};

    #[test]
    fn succeed_to_login_with_username_and_password() {
        // Given
        let mut srv = TestServer::build_with_state(State::default)
            .start(move |app| app.handler(|req| ws::start(req, WebSocket::new())));
        let (reader, mut writer) = srv.ws().unwrap();

        // When
        let mut builder = Builder::new_default();
        let mut data = Vec::new();
        {
            let mut creds = builder.init_root::<request::Builder>().init_login().init_credentials();
            creds.set_username("username");
            creds.set_password("username");
        }
        write_message(&mut data, &builder).unwrap();
        writer.binary(data);
        let (item, _) = srv.execute(reader.into_future()).unwrap();

        // Then
        match item {
            Some(Message::Binary(b)) => {
                let reader = read_message(&mut b.as_ref(), ReaderOptions::new()).unwrap();
                match reader.get_root::<response::Reader>().unwrap().which().unwrap() {
                    response::Login(d) => match d.which().unwrap() {
                        response::login::Token(_) => {}
                        _ => panic!("Wrong response content"),
                    },
                    _ => panic!("Wrong response type"),
                };
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn fail_to_login_with_wrong_username_and_password() {
        // Given
        let mut srv = TestServer::build_with_state(State::default)
            .start(move |app| app.handler(|req| ws::start(req, WebSocket::new())));
        let (reader, mut writer) = srv.ws().unwrap();

        // When
        let mut builder = Builder::new_default();
        let mut data = Vec::new();
        {
            let mut creds = builder.init_root::<request::Builder>().init_login().init_credentials();
            creds.set_username("username");
            creds.set_password("password");
        }
        write_message(&mut data, &builder).unwrap();
        writer.binary(data);
        let (item, _) = srv.execute(reader.into_future()).unwrap();

        // Then
        match item {
            Some(Message::Binary(b)) => {
                let reader = read_message(&mut b.as_ref(), ReaderOptions::new()).unwrap();
                match reader.get_root::<response::Reader>().unwrap().which().unwrap() {
                    response::Login(d) => match d.which().unwrap() {
                        response::login::Error(_) => {}
                        _ => panic!("Wrong response content"),
                    },
                    _ => panic!("Wrong response type"),
                };
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn succeed_to_login_with_token() {
        // Given
        let mut srv = TestServer::build_with_state(State::default)
            .start(move |app| app.handler(|req| ws::start(req, WebSocket::new())));
        let (reader, mut writer) = srv.ws().unwrap();

        // When
        let mut builder = Builder::new_default();
        let mut data = Vec::new();
        {
            let mut creds = builder.init_root::<request::Builder>().init_login().init_credentials();
            creds.set_username("username");
            creds.set_password("username");
        }
        write_message(&mut data, &builder).unwrap();
        writer.binary(data);
        let (item, _) = srv.execute(reader.into_future()).unwrap();
        let token = match item {
            Some(Message::Binary(b)) => {
                let reader = read_message(&mut b.as_ref(), ReaderOptions::new()).unwrap();
                match reader.get_root::<response::Reader>().unwrap().which().unwrap() {
                    response::Login(d) => match d.which().unwrap() {
                        response::login::Token(token) => token.unwrap().to_owned(),
                        _ => panic!("Wrong response content"),
                    },
                    _ => panic!("Wrong response type"),
                }
            }
            _ => panic!("Wrong message type"),
        };
        let mut token_data = Vec::new();
        builder.init_root::<request::Builder>().init_login().set_token(&token);
        write_message(&mut token_data, &builder).unwrap();
        let (reader, mut writer) = srv.ws().unwrap();
        writer.binary(token_data);
        let (token_item, _) = srv.execute(reader.into_future()).unwrap();

        // Then
        match token_item {
            Some(Message::Binary(b)) => {
                let reader = read_message(&mut b.as_ref(), ReaderOptions::new()).unwrap();
                match reader.get_root::<response::Reader>().unwrap().which().unwrap() {
                    response::Login(d) => match d.which().unwrap() {
                        response::login::Token(_) => {}
                        _ => panic!("Wrong response content"),
                    },
                    _ => panic!("Wrong response type"),
                }
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn fail_to_login_with_wrong_token() {
        // Given
        let mut srv = TestServer::build_with_state(State::default)
            .start(move |app| app.handler(|req| ws::start(req, WebSocket::new())));
        let (reader, mut writer) = srv.ws().unwrap();

        // When
        let mut builder = Builder::new_default();
        let mut data = Vec::new();
        let token = "wrong_token".to_owned();
        builder.init_root::<request::Builder>().init_login().set_token(&token);
        write_message(&mut data, &builder).unwrap();
        writer.binary(data);
        let (item, _) = srv.execute(reader.into_future()).unwrap();

        // Then
        match item {
            Some(Message::Binary(b)) => {
                let reader = read_message(&mut b.as_ref(), ReaderOptions::new()).unwrap();
                match reader.get_root::<response::Reader>().unwrap().which().unwrap() {
                    response::Login(d) => match d.which().unwrap() {
                        response::login::Error(_) => {}
                        _ => panic!("Wrong response content"),
                    },
                    _ => panic!("Wrong response type"),
                };
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn succeed_to_logout() {
        // Given
        let mut srv = TestServer::build_with_state(State::default)
            .start(move |app| app.handler(|req| ws::start(req, WebSocket::new())));
        let (reader, mut writer) = srv.ws().unwrap();

        // When
        let mut builder = Builder::new_default();
        let mut data = Vec::new();
        let token = "token".to_owned();
        builder.init_root::<request::Builder>().set_logout(&token);
        write_message(&mut data, &builder).unwrap();
        writer.binary(data);
        let (item, _) = srv.execute(reader.into_future()).unwrap();

        // Then
        match item {
            Some(Message::Binary(b)) => {
                let reader = read_message(&mut b.as_ref(), ReaderOptions::new()).unwrap();
                match reader.get_root::<response::Reader>().unwrap().which().unwrap() {
                    response::Logout(_) => {}
                    _ => panic!("Wrong response type"),
                };
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn succeed_to_close_the_connection() {
        // Given
        let mut srv = TestServer::build_with_state(State::default)
            .start(move |app| app.handler(|req| ws::start(req, WebSocket::new())));
        let (reader, mut writer) = srv.ws().unwrap();

        // When
        writer.close(None);

        // Then
        assert!(srv.execute(reader.into_future()).is_err());
    }
}
