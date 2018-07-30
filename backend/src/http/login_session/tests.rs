//! The session based login request tests

#![cfg(test)]

use actix::prelude::*;
use actix_web::test::TestServer;
use database::UpdateSession;
use failure::Error;
use http::{
    login_session::login_session,
    tests::{execute_request, state, DatabaseExecutorMock},
};
use serde_cbor::to_vec;
use token::Token;
use webapp::protocol::{model::Session, request};

impl Handler<UpdateSession> for DatabaseExecutorMock {
    type Result = Result<Session, Error>;

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
