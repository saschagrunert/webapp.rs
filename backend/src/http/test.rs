//! HTTP message test abstraction

#![cfg(test)]

use actix::prelude::*;

/// The mock database executor actor
pub struct DatabaseExecutorMock;

impl Actor for DatabaseExecutorMock {
    type Context = SyncContext<Self>;
}
