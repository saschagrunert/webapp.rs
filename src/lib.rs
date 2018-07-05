#![deny(missing_docs)]
//! The main library interface

extern crate capnp;
extern crate serde;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate failure;

#[macro_use]
extern crate lazy_static;

#[cfg(feature = "default")]
extern crate actix;

#[cfg(feature = "default")]
extern crate actix_web;

#[cfg(feature = "default")]
#[macro_use]
extern crate diesel;

#[cfg(feature = "default")]
extern crate futures;

#[cfg(feature = "default")]
#[macro_use]
extern crate log;

#[cfg(feature = "default")]
extern crate jsonwebtoken;

#[cfg(feature = "default")]
extern crate num_cpus;

#[cfg(feature = "default")]
extern crate openssl;

#[cfg(feature = "default")]
extern crate r2d2;

#[cfg(feature = "default")]
extern crate time;

#[cfg(feature = "default")]
extern crate uuid;

#[cfg(feature = "default")]
mod backend;

#[cfg(feature = "default")]
pub use backend::server::Server;

#[cfg(feature = "frontend")]
#[macro_use]
extern crate stdweb;

#[cfg(feature = "frontend")]
#[macro_use]
extern crate yew;

#[cfg(feature = "frontend")]
mod frontend;

#[cfg(feature = "frontend")]
pub use frontend::components::root::RootComponent;

pub mod protocol_capnp {
    #![allow(dead_code)]
    #![allow(missing_docs)]
    #![allow(unknown_lints)]
    #![allow(clippy)]
    include!(concat!(env!("OUT_DIR"), "/src/protocol_capnp.rs"));
}

pub mod config;

/// The global session cookie name
pub const SESSION_COOKIE: &str = "sessionToken";

/// The global config file name
pub const CONFIG_FILENAME: &str = "Config.toml";
