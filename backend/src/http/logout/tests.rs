//! The logout request test

#![cfg(test)]

use actix::prelude::*;
use actix_web::test::TestServer;
use database::DeleteSession;
use failure::Error;
use http::{
    logout::logout,
    tests::{execute_request, state, DatabaseExecutorMock},
};
use serde_cbor::to_vec;
use webapp::protocol::{model::Session, request};

impl Handler<DeleteSession> for DatabaseExecutorMock {
    type Result = Result<(), Error>;

    fn handle(&mut self, _: DeleteSession, _: &mut Self::Context) -> Self::Result {
        Ok(())
    }
}

fn create_testserver() -> TestServer {
    TestServer::build_with_state(state).start(|app| app.handler(logout))
}

#[test]
fn succeed_to_logout() {
    // Given
    let mut server = create_testserver();
    let body = to_vec(&request::Logout(Session {
        token: "any-token".to_owned(),
    })).unwrap();

    // When
    let response = execute_request(&mut server, body);

    // Then
    assert!(response.status().is_success());
}
