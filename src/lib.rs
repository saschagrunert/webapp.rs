#![deny(missing_docs)]
//! The main library interface

#[macro_use]
extern crate yew;

mod client;

pub use client::root::{Context, RootComponent};
