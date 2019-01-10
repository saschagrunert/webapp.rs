//! The logout request test

#![cfg(test)]

use crate::{
    database::DeleteSession,
    http::{
        logout::logout,
        test::{execute_request, state, DatabaseExecutorMock},
    },
};
use actix::prelude::*;
use actix_web::test::TestServer;
use failure::Fallible;
use serde_cbor::to_vec;
use serde_derive::Serialize;
use webapp::protocol::{model::Session, request};

impl Handler<DeleteSession> for DatabaseExecutorMock {
    type Result = Fallible<()>;

    fn handle(&mut self, _: DeleteSession, _: &mut Self::Context) -> Self::Result {
        Ok(())
    }
}

fn create_testserver() -> TestServer {
    TestServer::build_with_state(state).start(|app| app.handler(logout))
}

#[test]
fn succeed_to_logout() -> Fallible<()> {
    // Given
    let mut server = create_testserver();
    let body = to_vec(&request::Logout(Session::new("any-token")))?;

    // When
    let response = execute_request(&mut server, body)?;

    // Then
    assert!(response.status().is_success());
    Ok(())
}

#[test]
fn fail_to_logout_with_invalid_cbor() -> Fallible<()> {
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
