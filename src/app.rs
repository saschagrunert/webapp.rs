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
pub async fn login(username: String, password: String) -> Result<String, ServerFnError> {
    use crate::{auth, database};

    if username.is_empty() || password.is_empty() || username != password {
        return Err(ServerFnError::new("Invalid credentials"));
    }

    let token = auth::create_token(&username).map_err(|e| ServerFnError::new(e.to_string()))?;
    database::create_session(&token)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(token)
}

#[server]
pub async fn renew_session(token: String) -> Result<String, ServerFnError> {
    use crate::{auth, database};

    let username = auth::verify_token(&token).map_err(|e| ServerFnError::new(e.to_string()))?;

    let new_token = auth::create_token(&username).map_err(|e| ServerFnError::new(e.to_string()))?;
    database::update_session(&token, &new_token)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(new_token)
}

#[server]
pub async fn logout(token: String) -> Result<(), ServerFnError> {
    use crate::database;

    database::delete_session(&token)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}
