//! The frontend library

#![deny(missing_docs)]
#![recursion_limit = "512"]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate stdweb;
extern crate webapp;
#[macro_use]
extern crate yew;

#[macro_use]
mod api;
mod component;
mod route;
mod service;
mod string;

pub use component::root::RootComponent;
pub use service::log::init_logger;

/// The global session cookie name
const SESSION_COOKIE: &str = "sessionToken";
