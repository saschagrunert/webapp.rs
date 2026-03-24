pub mod app;
#[cfg(feature = "ssr")]
pub mod auth;
#[cfg(feature = "ssr")]
pub mod csrf;
#[cfg(feature = "ssr")]
pub mod database;
pub mod pages;
#[cfg(feature = "ssr")]
pub mod rate_limit;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(app::App);
}
