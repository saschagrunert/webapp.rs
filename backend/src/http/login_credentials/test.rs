//! The credential based login request tests

#![cfg(test)]

use crate::{
    database::CreateSession,
    http::{
        login_credentials::login_credentials,
        test::{execute_request, state, DatabaseExecutorMock},
    },
    token::Token,
};
use actix::prelude::*;
use actix_web::test::TestServer;
use failure::Fallible;
use serde_cbor::to_vec;
use serde_derive::Serialize;
use webapp::protocol::{model::Session, request::LoginCredentials};

impl Handler<CreateSession> for DatabaseExecutorMock {
    type Result = Fallible<Session>;

    fn handle(&mut self, _: CreateSession, _: &mut Self::Context) -> Self::Result {
        Ok(Session::new(Token::create("username").unwrap()))
    }
}

fn create_testserver() -> TestServer {
    TestServer::build_with_state(state).start(|app| app.handler(login_credentials))
}

#[test]
fn succeed_to_login_with_credentials() -> Fallible<()> {
    // Given
    let mut server = create_testserver();
    let body = to_vec(&LoginCredentials {
        username: "username".to_owned(),
        password: "username".to_owned(),
    })?;

    // When
    let response = execute_request(&mut server, body)?;

    // Then
    assert!(response.status().is_success());
    Ok(())
}

#[test]
fn fail_to_login_with_wrong_credentials() -> Fallible<()> {
    // Given
    let mut server = create_testserver();
    let body = to_vec(&LoginCredentials {
        username: "username".to_owned(),
        password: "password".to_owned(),
    })?;

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
