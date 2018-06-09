#![deny(missing_docs)]
//! The main library interface

#[macro_use]
extern crate serde_derive;
extern crate failure;
extern crate serde_json;

#[cfg(feature = "frontend")]
extern crate stdweb;

#[cfg(feature = "frontend")]
#[macro_use]
extern crate yew;

#[cfg(feature = "frontend")]
mod frontend;

#[cfg(feature = "frontend")]
pub use frontend::root::{Context, RootComponent};

#[cfg(feature = "backend")]
extern crate tungstenite;

#[cfg(feature = "backend")]
mod backend;

#[cfg(feature = "backend")]
pub use backend::Server;

mod shared;

/// The global API url for websocket communication
pub const API_URL: &str = "ws://saschagrunert.de:30000";
