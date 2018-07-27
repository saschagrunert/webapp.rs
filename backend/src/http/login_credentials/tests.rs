//! The credential based login request tests

#![cfg(test)]

use actix::prelude::*;
use actix_web::test::TestServer;
use database::{CreateSession, DatabaseError};
use http::{
    login_credentials::login_credentials,
    tests::{execute_request, state, DatabaseExecutorMock},
};
use serde_cbor::to_vec;
use token::Token;
use webapp::protocol::{model::Session, request};

impl Handler<CreateSession> for DatabaseExecutorMock {
    type Result = Result<Session, DatabaseError>;
    fn handle(&mut self, _: CreateSession, _: &mut Self::Context) -> Self::Result {
        Ok(Session {
            token: Token::create("username").unwrap(),
        })
    }
}

fn create_testserver() -> TestServer {
    TestServer::build_with_state(state).start(|app| app.handler(login_credentials))
}

#[test]
fn succeed_to_login_with_credentials() {
    // Given
    let mut server = create_testserver();
    let body = to_vec(&request::LoginCredentials {
        username: "username".to_owned(),
        password: "username".to_owned(),
    }).unwrap();

    // When
    let response = execute_request(&mut server, body);

    // Then
    assert!(response.status().is_success());
}

#[test]
fn fail_to_login_with_wrong_credentials() {
    // Given
    let mut server = create_testserver();
    let body = to_vec(&request::LoginCredentials {
        username: "username".to_owned(),
        password: "password".to_owned(),
    }).unwrap();

    // When
    let response = execute_request(&mut server, body);

    // Then
    assert_eq!(response.status().is_success(), false);
}
