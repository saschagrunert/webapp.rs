//! The frontend library
#![deny(missing_docs)]
#![recursion_limit = "512"]

#[macro_use]
mod api;
mod component;
mod route;
mod service;
mod string;

pub use crate::{component::root::RootComponent, service::log::init_logger};

/// The global session cookie name
const SESSION_COOKIE: &str = "sessionToken";
