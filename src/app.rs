use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};

use crate::pages::{content::ContentPage, login::LoginPage};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/webapp.css"/>
        <Title text="WebApp.rs"/>
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=LoginPage/>
                    <Route path=StaticSegment("content") view=ContentPage/>
                </Routes>
            </main>
        </Router>
    }
}

#[server]
pub async fn register(username: String, password: String) -> Result<(), ServerFnError> {
    use crate::{auth, database};

    if username.is_empty() || password.is_empty() {
        return Err(ServerFnError::new("Username and password are required"));
    }

    if username.len() > 64 || password.len() > 128 {
        return Err(ServerFnError::new("Input too long"));
    }

    if database::user_exists(&username)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        return Err(ServerFnError::new("User already exists"));
    }

    let hash = auth::hash_password(&password).map_err(ServerFnError::new)?;
    database::create_user(&username, &hash)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

#[server]
pub async fn login(username: String, password: String) -> Result<String, ServerFnError> {
    use crate::{auth, database};

    if username.is_empty() || password.is_empty() {
        return Err(ServerFnError::new("Invalid credentials"));
    }

    let hash = database::get_password_hash(&username)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Invalid credentials"))?;

    if !auth::verify_password(&password, &hash).map_err(ServerFnError::new)? {
        return Err(ServerFnError::new("Invalid credentials"));
    }

    let token = auth::create_token(&username).map_err(|e| ServerFnError::new(e.to_string()))?;
    let expires_at = auth::token_expiry();
    database::create_session(&token, &username, expires_at)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(token)
}

#[server]
pub async fn renew_session(token: String) -> Result<String, ServerFnError> {
    use crate::{auth, database};

    let username = auth::verify_token(&token).map_err(|e| ServerFnError::new(e.to_string()))?;

    if !database::session_exists(&token)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        return Err(ServerFnError::new("Session not found"));
    }

    let new_token = auth::create_token(&username).map_err(|e| ServerFnError::new(e.to_string()))?;
    let expires_at = auth::token_expiry();
    database::update_session(&token, &new_token, expires_at)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(new_token)
}

#[server]
pub async fn logout(token: String) -> Result<(), ServerFnError> {
    use crate::{auth, database};

    // Verify the token is valid before attempting deletion
    auth::verify_token(&token).map_err(|e| ServerFnError::new(e.to_string()))?;

    if !database::delete_session(&token)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        return Err(ServerFnError::new("Session not found"));
    }

    Ok(())
}
