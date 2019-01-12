//! HTTP message test abstraction

#![cfg(test)]

use crate::server::State;
use actix::prelude::*;
use actix_web::{client::ClientResponse, test::TestServer};
use failure::{format_err, Fallible};

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

pub fn execute_request(server: &mut TestServer, body: Vec<u8>) -> Fallible<ClientResponse> {
    let request = server
        .post()
        .body(body)
        .map_err(|_| format_err!("Unable to create post request"))?;
    Ok(server.execute(request.send())?)
}
