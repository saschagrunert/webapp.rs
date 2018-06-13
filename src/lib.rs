#![deny(missing_docs)]
//! The main library interface

#[macro_use]
extern crate serde_derive;
extern crate failure;
extern crate serde_json;

#[cfg(feature = "default")]
extern crate actix;

#[cfg(feature = "default")]
extern crate actix_web;

#[cfg(feature = "default")]
#[macro_use]
extern crate log;

#[cfg(feature = "default")]
mod backend;

#[cfg(feature = "default")]
pub use backend::Server;

#[cfg(feature = "frontend")]
extern crate stdweb;

#[cfg(feature = "frontend")]
#[macro_use]
extern crate yew;

#[cfg(feature = "frontend")]
mod frontend;

#[cfg(feature = "frontend")]
pub use frontend::root::{Context, RootComponent};

mod shared;

/// The global API url for websocket communication
pub const API_URL: &str = "ws://localhost:30000";
