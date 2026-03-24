use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

use crate::app::logout;
use crate::pages::login::{get_cookie, remove_cookie};

#[cfg(feature = "hydrate")]
use crate::app::renew_session;
#[cfg(feature = "hydrate")]
use crate::pages::login::set_cookie;

#[component]
pub fn ContentPage() -> impl IntoView {
    let navigate = use_navigate();
    let token = RwSignal::new(String::new());
    let logging_out = RwSignal::new(false);

    // Check session on mount
    Effect::new({
        let navigate = navigate.clone();
        move |_| match get_cookie("session_token") {
            Some(t) => token.set(t),
            None => navigate("/", Default::default()),
        }
    });

    // Session renewal every 30 seconds
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::{JsCast, closure::Closure};

        let cb = Closure::wrap(Box::new(move || {
            let current = token.get_untracked();
            if current.is_empty() {
                return;
            }
            spawn_local(async move {
                if let Ok(new_token) = renew_session(current).await {
                    set_cookie("session_token", &new_token);
                    token.set(new_token);
                }
            });
        }) as Box<dyn Fn()>);

        let window = web_sys::window().unwrap();
        let interval_id = window
            .set_interval_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                30_000,
            )
            .unwrap();
        cb.forget();

        on_cleanup(move || {
            if let Some(window) = web_sys::window() {
                window.clear_interval_with_handle(interval_id);
            }
        });
    }

    let on_logout = move |_| {
        let navigate = navigate.clone();
        logging_out.set(true);
        let current_token = token.get();

        spawn_local(async move {
            let _ = logout(current_token).await;
            remove_cookie("session_token");
            navigate("/", Default::default());
        });
    };

    view! {
        <div class="container">
            <div class="card">
                <h1>"Welcome"</h1>
                <p>"You are logged in to a web application completely written in Rust."</p>
                <button on:click=on_logout disabled=move || logging_out.get()>
                    {move || if logging_out.get() { "Logging out..." } else { "Logout" }}
                </button>
            </div>
        </div>
    }
}
