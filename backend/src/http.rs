//! HTTP message handling

use actix::{dev::ToEnvelope, prelude::*};
use actix_web::{
    error::{Error as HttpError, ErrorInternalServerError, ErrorUnauthorized},
    AsyncResponder, HttpRequest, HttpResponse,
};
use cbor::{CborRequest, CborResponseBuilder};
use database::{CreateSession, DeleteSession, UpdateSession};
use futures::Future;
use server::State;
use token::Token;
use webapp::protocol::{model::Session, request, response};

type FutureResponse = Box<Future<Item = HttpResponse, Error = HttpError>>;

pub fn login_credentials<T>(http_request: &HttpRequest<State<T>>) -> FutureResponse
where
    T: Actor + Handler<CreateSession>,
    <T as Actor>::Context: ToEnvelope<T, CreateSession>,
{
    let request_clone = http_request.clone();
    CborRequest::new(http_request)
        .from_err()
        // Verify username and password
        .and_then(|request::LoginCredentials{username, password}| {
            debug!("User {} is trying to login", username);
            if username.is_empty() || password.is_empty() || username != password {
                return Err(ErrorUnauthorized("wrong username or password"));
            }
            Ok(username)
        })
        // Create a new token
        .and_then(|username| {
            Token::create(&username).map_err(|_| {
                 ErrorInternalServerError("token creation failed")
            })
        })
        // Update the session in the database
        .and_then(move |token| {
            request_clone
                .state()
                .database
                .send(CreateSession(token))
                .from_err()
                .and_then(|result| match result {
                    Ok(r) => Ok(HttpResponse::Ok().cbor(response::Login(r))?),
                    Err(_) => Ok(HttpResponse::InternalServerError().into()),
                })
        })
        .responder()
}

pub fn login_session<T>(http_request: &HttpRequest<State<T>>) -> FutureResponse
where
    T: Actor + Handler<UpdateSession>,
    <T as Actor>::Context: ToEnvelope<T, UpdateSession>,
{
    let request_clone = http_request.clone();
    CborRequest::new(http_request)
        .from_err()
        // Create a new token for the already given one
        .and_then(|request::LoginSession(Session{token})| {
            debug!("Session token {} wants to be renewed", token);
            Token::verify(&token).map_err(|_| {
                 ErrorUnauthorized("Token verification failed")
            }).and_then(|new_token| {
                 Ok((token, new_token))
            })
        })
        // Update the session in the database
        .and_then(move |(old_token, new_token)| {
            request_clone
                .state()
                .database
                .send(UpdateSession { old_token, new_token })
                .from_err()
                .and_then(|result| match result {
                    Ok(r) => Ok(HttpResponse::Ok().cbor(response::Login(r))?),
                    Err(_) => Ok(HttpResponse::InternalServerError().into()),
                })
        })
        .responder()
}

pub fn logout<T: Actor>(http_request: &HttpRequest<State<T>>) -> FutureResponse
where
    T: Actor + Handler<DeleteSession>,
    <T as Actor>::Context: ToEnvelope<T, DeleteSession>,
{
    let request_clone = http_request.clone();
    CborRequest::new(http_request)
        .from_err()
        // Remove the session from the database
        .and_then(move |request::Logout(Session{token})| {
            debug!("Session token {} wants to be logged out", token);
            request_clone
                .state()
                .database
                .send(DeleteSession(token))
                .from_err()
                .and_then(|result| match result {
                    Ok(()) => Ok(HttpResponse::Ok().cbor(response::Logout)?),
                    Err(_) => Ok(HttpResponse::InternalServerError().into()),
                })
        })
        .responder()
}

#[cfg(test)]
mod tests {
    use serde_cbor::to_vec;
    use super::*;
    use database::DatabaseError;
    use actix_web::{client::ClientResponse, http::{StatusCode}, test::TestServer};

    /// The mock database executor actor
    pub struct DatabaseExecutorMock;

    impl Actor for DatabaseExecutorMock {
        type Context = SyncContext<Self>;
    }

    impl Handler<CreateSession> for DatabaseExecutorMock {
        type Result = Result<Session, DatabaseError>;
        fn handle(&mut self, _: CreateSession, _: &mut Self::Context) -> Self::Result {
            Ok(Session { token: "token".to_owned() })
        }
    }

    fn create_testserver() -> TestServer {
        TestServer::build_with_state(|| {
            State { database: SyncArbiter::start(1, move || DatabaseExecutorMock) }
        }).start(|app| app.handler(login_credentials))
    }

    fn execute_request(server: &mut TestServer, body: Vec<u8>) -> ClientResponse {
        let request = server.post().body(body).unwrap();
        server.execute(request.send()).unwrap()
    }

    #[test]
    fn succeed_to_login_with_credentials() {
        // Given
        let mut server = create_testserver();

        // When
        let body = to_vec(&request::LoginCredentials {
            username: "username".to_owned(),
            password: "username".to_owned(),
        }).unwrap();
        let response = execute_request(&mut server, body);

        // Then
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn fail_to_login_with_credentials_wrong() {
        // Given
        let mut server = create_testserver();

        // When
        let body = to_vec(&request::LoginCredentials {
            username: "username".to_owned(),
            password: "password".to_owned(),
        }).unwrap();
        let response = execute_request(&mut server, body);

        // Then
        assert!(!response.status().is_success());
    }
}
