//! The main library interface
#![deny(missing_docs)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;
extern crate serde_cbor;

pub mod config;
pub mod protocol;

/// The global config file name
pub const CONFIG_FILENAME: &str = "Config.toml";
