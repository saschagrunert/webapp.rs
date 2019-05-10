//! The core library
#![deny(missing_docs)]
#![allow(unknown_lints, proc_macro_derive_resolution_fallback)]

#[cfg(feature = "backend")]
#[macro_use]
extern crate diesel;

pub mod config;
pub mod protocol;
#[cfg(feature = "backend")]
#[allow(missing_docs)]
pub mod schema;

/// The global config file name
pub const CONFIG_FILENAME: &str = "Config.toml";

macro_rules! apis {
    ($($name:ident => $content:expr,)*) => (
        $(#[allow(missing_docs)] pub const $name: &str = $content;)*
    )
}

apis! {
    API_URL_LOGIN_CREDENTIALS => "login/credentials",
    API_URL_LOGIN_SESSION => "login/session",
    API_URL_LOGOUT => "logout",
}
