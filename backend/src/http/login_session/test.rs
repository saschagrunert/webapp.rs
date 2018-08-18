//! The session based login request tests

#![cfg(test)]

use actix::prelude::*;
use actix_web::test::TestServer;
use database::UpdateSession;
use failure::Fallible;
use http::{
    login_session::login_session,
    test::{execute_request, state, DatabaseExecutorMock},
};
use serde_cbor::to_vec;
use token::Token;
use webapp::protocol::{model::Session, request};

impl Handler<UpdateSession> for DatabaseExecutorMock {
    type Result = Fallible<Session>;

    fn handle(&mut self, _: UpdateSession, _: &mut Self::Context) -> Self::Result {
        Ok(Session::new(Token::create("username").unwrap()))
    }
}

fn create_testserver() -> TestServer {
    TestServer::build_with_state(state).start(|app| app.handler(login_session))
}

#[test]
fn succeed_to_login_with_session() -> Fallible<()> {
    // Given
    let mut server = create_testserver();
    let token = Token::create("username")?;
    let body = to_vec(&request::LoginSession(Session::new(token)))?;

    // When
    let response = execute_request(&mut server, body)?;

    // Then
    assert!(response.status().is_success());
    Ok(())
}

#[test]
fn fail_to_login_with_wrong_session() -> Fallible<()> {
    // Given
    let mut server = create_testserver();
    let body = to_vec(&request::LoginSession(Session::new("wrong")))?;

    // When
    let response = execute_request(&mut server, body)?;

    // Then
    assert_eq!(response.status().is_success(), false);
    Ok(())
}

#[test]
fn fail_to_login_with_invalid_cbor() -> Fallible<()> {
    // Given
    #[derive(Serialize)]
    struct Invalid;
    let mut server = create_testserver();
    let body = to_vec(&Invalid)?;

    // When
    let response = execute_request(&mut server, body)?;

    // Then
    assert_eq!(response.status().is_success(), false);
    Ok(())
}
