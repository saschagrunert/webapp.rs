#![deny(missing_docs)]
//! The main library interface

extern crate capnp;

#[macro_use]
extern crate failure;

#[cfg(feature = "default")]
extern crate actix;

#[cfg(feature = "default")]
extern crate actix_web;

#[cfg(feature = "default")]
#[macro_use]
extern crate log;

#[cfg(feature = "default")]
extern crate openssl;

#[cfg(feature = "default")]
mod backend;

#[cfg(feature = "default")]
pub use backend::Server;

#[cfg(feature = "frontend")]
#[macro_use]
extern crate stdweb;

#[cfg(feature = "frontend")]
#[macro_use]
extern crate yew;

#[cfg(feature = "frontend")]
mod frontend;

#[cfg(feature = "frontend")]
pub use frontend::root::RootComponent;

pub mod protocol_capnp {
    #![allow(dead_code)]
    #![allow(missing_docs)]
    #![allow(unknown_lints)]
    #![allow(clippy)]
    include!(concat!(env!("OUT_DIR"), "/src/protocol_capnp.rs"));
}

/// The global API url for websocket communication
pub const API_URL: &str = "wss://saschagrunert.de:30000/ws";

/// The global session cookie name
pub const SESSION_COOKIE: &str = "sessionToken";
