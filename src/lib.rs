//! The main library interface

#![deny(missing_docs)]
#![allow(unknown_lints, proc_macro_derive_resolution_fallback)]

#[cfg(feature = "backend")]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;

pub mod config;
pub mod protocol;
#[cfg(feature = "backend")]
#[allow(missing_docs)]
pub mod schema;

/// The global config file name
pub const CONFIG_FILENAME: &str = "Config.toml";

/// The API URL for login with credentials
pub const API_URL_LOGIN_CREDENTIALS: &str = "login/credentials";

/// The API URL for login with session
pub const API_URL_LOGIN_SESSION: &str = "login/session";

/// The API URL for logout
pub const API_URL_LOGOUT: &str = "logout";
