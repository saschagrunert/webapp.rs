use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

use crate::app::login;

pub fn get_cookie(name: &str) -> Option<String> {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::JsCast;
        let document = web_sys::window()?.document()?;
        let html_doc: web_sys::HtmlDocument = document.dyn_into().ok()?;
        let cookies = html_doc.cookie().ok()?;
        cookies
            .split(';')
            .filter_map(|c| {
                let (key, value) = c.trim().split_once('=')?;
                (key == name).then(|| value.to_owned())
            })
            .next()
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = name;
        None
    }
}

pub fn set_cookie(name: &str, value: &str) {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::JsCast;
        if let Some(Ok(html_doc)) = web_sys::window()
            .and_then(|w| w.document())
            .map(|d| d.dyn_into::<web_sys::HtmlDocument>())
        {
            let _ = html_doc.set_cookie(&format!(
                "{name}={value}; path=/; max-age=86400; SameSite=Strict"
            ));
        }
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (name, value);
    }
}

pub fn remove_cookie(name: &str) {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::JsCast;
        if let Some(Ok(html_doc)) = web_sys::window()
            .and_then(|w| w.document())
            .map(|d| d.dyn_into::<web_sys::HtmlDocument>())
        {
            let _ = html_doc.set_cookie(&format!("{name}=; path=/; max-age=0"));
        }
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = name;
    }
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error = RwSignal::new(Option::<String>::None);
    let pending = RwSignal::new(false);
    let navigate = use_navigate();

    // Check for existing session on mount
    Effect::new({
        let navigate = navigate.clone();
        move |_| {
            if get_cookie("session_token").is_some() {
                navigate("/content", Default::default());
            }
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let navigate = navigate.clone();
        pending.set(true);
        error.set(None);

        spawn_local(async move {
            match login(username.get(), password.get()).await {
                Ok(token) => {
                    set_cookie("session_token", &token);
                    navigate("/content", Default::default());
                }
                Err(_) => {
                    error.set(Some("Authentication failed".into()));
                    pending.set(false);
                }
            }
        });
    };

    let disabled = move || pending.get() || username.get().is_empty() || password.get().is_empty();

    view! {
        <div class="container">
            <div class="card">
                <h1>"WebApp.rs"</h1>
                <p class="subtitle">"A web application completely written in Rust"</p>
                <form on:submit=on_submit>
                    <div class="field">
                        <input
                            type="text"
                            placeholder="Username"
                            prop:value=username
                            on:input=move |ev| username.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="field">
                        <input
                            type="password"
                            placeholder="Password"
                            prop:value=password
                            on:input=move |ev| password.set(event_target_value(&ev))
                        />
                    </div>
                    {move || {
                        error
                            .get()
                            .map(|msg| {
                                view! { <div class="error">{msg}</div> }
                            })
                    }}
                    <button type="submit" disabled=disabled>
                        {move || if pending.get() { "Logging in..." } else { "Login" }}
                    </button>
                </form>
            </div>
        </div>
    }
}
