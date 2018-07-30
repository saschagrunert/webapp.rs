//! HTTP message test abstraction

#![cfg(test)]

use actix::prelude::*;
use actix_web::{client::ClientResponse, test::TestServer};
use server::State;

/// The mock database executor actor
pub struct DatabaseExecutorMock;

impl Actor for DatabaseExecutorMock {
    type Context = SyncContext<Self>;
}

pub fn state() -> State<DatabaseExecutorMock> {
    State {
        database: SyncArbiter::start(1, move || DatabaseExecutorMock),
    }
}

pub fn execute_request(server: &mut TestServer, body: Vec<u8>) -> ClientResponse {
    let request = server.post().body(body).unwrap();
    server.execute(request.send()).unwrap()
}
