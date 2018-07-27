//! The session based login request

use actix::{dev::ToEnvelope, prelude::*};
use actix_web::{error::ErrorUnauthorized, AsyncResponder, HttpRequest, HttpResponse};
use cbor::{CborRequest, CborResponseBuilder};
use database::UpdateSession;
use futures::Future;
use http::FutureResponse;
use server::State;
use token::Token;
use webapp::protocol::{model::Session, request, response};

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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestServer;
    use database::DatabaseError;
    use http::tests::{execute_request, state, DatabaseExecutorMock};
    use serde_cbor::to_vec;
    use webapp::protocol::{model::Session, request};

    impl Handler<UpdateSession> for DatabaseExecutorMock {
        type Result = Result<Session, DatabaseError>;
        fn handle(&mut self, _: UpdateSession, _: &mut Self::Context) -> Self::Result {
            Ok(Session {
                token: Token::create("username").unwrap(),
            })
        }
    }

    fn create_testserver() -> TestServer {
        TestServer::build_with_state(state).start(|app| app.handler(login_session))
    }

    #[test]
    fn succeed_to_login_with_session() {
        // Given
        let mut server = create_testserver();
        let token = Token::create("username").unwrap();
        let body = to_vec(&request::LoginSession(Session { token })).unwrap();

        // When
        let response = execute_request(&mut server, body);

        // Then
        assert!(response.status().is_success());
    }

    #[test]
    fn fail_to_login_with_wrong_session() {
        // Given
        let mut server = create_testserver();
        let body = to_vec(&request::LoginSession(Session {
            token: "wrong".to_owned(),
        })).unwrap();

        // When
        let response = execute_request(&mut server, body);

        // Then
        assert_eq!(response.status().is_success(), false);
    }
}
