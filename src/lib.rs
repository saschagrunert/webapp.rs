#![deny(missing_docs)]
//! The main library interface

extern crate failure;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate yew;

mod client;

pub use client::root::{Context, RootComponent};
