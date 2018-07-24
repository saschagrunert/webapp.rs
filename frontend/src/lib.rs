//! The main library interface

#![deny(missing_docs)]
#![recursion_limit = "128"]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate stdweb;
#[macro_use]
extern crate yew;
extern crate serde;
extern crate serde_cbor;
extern crate webapp;

mod component;
mod route;
mod service;
mod string;

pub use component::root::RootComponent;

/// The global session cookie name
pub const SESSION_COOKIE: &str = "sessionToken";

/// The API URL for the login credential request
pub const API_URL_LOGIN_SESSION: &str = concat!(env!("API_URL"), "login/session");
