//! The main library interface
#![deny(missing_docs)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
extern crate actix;
extern crate actix_web;
extern crate diesel;
extern crate failure;
extern crate futures;
extern crate jsonwebtoken;
extern crate num_cpus;
extern crate openssl;
extern crate r2d2;
extern crate serde;
extern crate serde_cbor;
extern crate time;
extern crate uuid;
extern crate webapp;

mod database;
mod server;
mod token;
mod websocket;

pub use server::Server;
