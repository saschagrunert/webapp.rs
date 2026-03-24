#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::{Router, http::StatusCode, middleware, routing::get};
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use tower_http::compression::CompressionLayer;
    use tracing_subscriber::{EnvFilter, fmt};
    use webapp::app::*;

    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    webapp::database::init()
        .await
        .expect("failed to initialize database");

    let conf = get_configuration(None).expect("failed to load leptos configuration");
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    let app = Router::new()
        .route(
            "/healthz",
            get(|| async {
                match sqlx::query("SELECT 1")
                    .execute(webapp::database::pool())
                    .await
                {
                    Ok(_) => StatusCode::OK,
                    Err(_) => StatusCode::SERVICE_UNAVAILABLE,
                }
            }),
        )
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(middleware::from_fn(webapp::csrf::validate))
        .layer(middleware::from_fn(webapp::rate_limit::check))
        .layer(CompressionLayer::new())
        .with_state(leptos_options);

    // Periodically clean up expired sessions every 5 minutes
    tokio::spawn(async {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
        loop {
            interval.tick().await;
            match webapp::database::delete_expired_sessions().await {
                Ok(0) => {}
                Ok(n) => tracing::info!("cleaned up {n} expired sessions"),
                Err(e) => tracing::warn!("failed to clean up expired sessions: {e}"),
            }
        }
    });

    tracing::info!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind to address");
    axum::serve(listener, app.into_make_service())
        .await
        .expect("server error");
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
