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
