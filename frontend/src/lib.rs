//! The frontend library
#![deny(missing_docs)]
#![recursion_limit = "512"]

use wasm_bindgen::prelude::*;

#[macro_use]
mod api;
mod component;
mod route;
mod service;
mod string;

use crate::{component::root::RootComponent, service::log::init_logger};

/// The global session cookie name
const SESSION_COOKIE: &str = "sessionToken";

/// Start the application
#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    // Initialize the logger
    init_logger().map_err(|e| JsValue::from(e.to_string()))?;

    // Run the application
    yew::start_app::<RootComponent>();
    Ok(())
}
