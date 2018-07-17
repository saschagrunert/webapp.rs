//! The main library interface
#![deny(missing_docs)]

#[macro_use] extern crate diesel;
#[macro_use] extern crate failure;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;
extern crate actix;
extern crate actix_web;
extern crate futures;
extern crate jsonwebtoken;
extern crate num_cpus;
extern crate openssl;
extern crate r2d2;
extern crate serde;
extern crate serde_cbor;
extern crate time;
extern crate uuid;

pub use server::Server;

mod token;
mod websocket;
pub mod database;
pub mod server;

/// The global session cookie name
pub const SESSION_COOKIE: &str = "sessionToken";

/// The global config file name
pub const CONFIG_FILENAME: &str = "Config.toml";
