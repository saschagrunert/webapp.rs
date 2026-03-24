use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::Mutex,
    time::{Duration, Instant},
};

const MAX_REQUESTS: usize = 20;
const WINDOW: Duration = Duration::from_secs(60);

static CLIENTS: Mutex<Option<HashMap<IpAddr, Vec<Instant>>>> = Mutex::new(None);

fn is_allowed(ip: IpAddr) -> bool {
    let mut guard = CLIENTS.lock().unwrap_or_else(|e| e.into_inner());
    let map = guard.get_or_insert_with(HashMap::new);
    let now = Instant::now();
    // Evict IPs with no recent requests to prevent unbounded growth
    map.retain(|_, v| {
        v.retain(|t| now.duration_since(*t) < WINDOW);
        !v.is_empty()
    });
    let timestamps = map.entry(ip).or_default();
    if timestamps.len() >= MAX_REQUESTS {
        false
    } else {
        timestamps.push(now);
        true
    }
}

fn extract_ip(req: &Request<Body>) -> IpAddr {
    req.headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse::<IpAddr>().ok())
        .unwrap_or(IpAddr::from([127, 0, 0, 1]))
}

pub async fn check(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    if req.method() != Method::POST {
        return Ok(next.run(req).await);
    }

    if is_allowed(extract_ip(&req)) {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::TOO_MANY_REQUESTS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, middleware, routing::get, routing::post};
    use tower::ServiceExt;

    fn app() -> Router {
        Router::new()
            .route("/test", post(|| async { "ok" }))
            .route("/get", get(|| async { "ok" }))
            .layer(middleware::from_fn(check))
    }

    #[tokio::test]
    async fn get_requests_bypass_rate_limit() {
        let resp = app()
            .oneshot(Request::get("/get").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn post_requests_are_rate_limited() {
        // Use a unique IP to avoid interference from other tests
        let ip = "10.99.99.99";
        for i in 0..MAX_REQUESTS {
            let resp = app()
                .oneshot(
                    Request::post("/test")
                        .header("x-forwarded-for", ip)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(resp.status(), StatusCode::OK, "request {i} should succeed");
        }

        // Next request should be rate limited
        let resp = app()
            .oneshot(
                Request::post("/test")
                    .header("x-forwarded-for", ip)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn extract_ip_from_forwarded_header() {
        let req = Request::post("/test")
            .header("x-forwarded-for", "192.168.1.1, 10.0.0.1")
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_ip(&req), IpAddr::from([192, 168, 1, 1]));
    }

    #[test]
    fn extract_ip_default_without_header() {
        let req = Request::post("/test").body(Body::empty()).unwrap();
        assert_eq!(extract_ip(&req), IpAddr::from([127, 0, 0, 1]));
    }

    #[test]
    fn is_allowed_enforces_limit() {
        let ip = IpAddr::from([10, 88, 88, 88]);
        for _ in 0..MAX_REQUESTS {
            assert!(is_allowed(ip));
        }
        assert!(!is_allowed(ip));
    }
}
