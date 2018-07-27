//! The main library interface

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

mod component;
mod route;
mod service;
mod string;

pub use component::root::RootComponent;
pub use service::log::init_logger;

/// The global session cookie name
const SESSION_COOKIE: &str = "sessionToken";

/// The API URLs
const API_URL_LOGIN_CREDENTIALS: &str = env!("API_URL_LOGIN_CREDENTIALS");
const API_URL_LOGIN_SESSION: &str = env!("API_URL_LOGIN_SESSION");
const API_URL_LOGOUT: &str = env!("API_URL_LOGOUT");
