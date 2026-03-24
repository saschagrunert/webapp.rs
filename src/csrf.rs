use axum::{
    body::Body,
    http::{Method, Request, StatusCode, header},
    middleware::Next,
    response::Response,
};

pub async fn validate(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    if req.method() != Method::POST {
        return Ok(next.run(req).await);
    }

    let host = req
        .headers()
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    let origin = req
        .headers()
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok());

    let referer = req
        .headers()
        .get(header::REFERER)
        .and_then(|v| v.to_str().ok());

    let valid = match (origin, referer) {
        (Some(origin), _) => origin_matches_host(origin, host),
        (None, Some(referer)) => origin_matches_host(referer, host),
        (None, None) => false,
    };

    if valid {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

fn origin_matches_host(origin: &str, host: &str) -> bool {
    // Extract host portion from origin URL (e.g., "http://localhost:3000" -> "localhost:3000")
    origin
        .split("//")
        .nth(1)
        .map(|h| h.split('/').next().unwrap_or(h))
        .is_some_and(|origin_host| origin_host == host)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, middleware, routing::get, routing::post};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn app() -> Router {
        Router::new()
            .route("/test", post(|| async { "ok" }))
            .route("/get", get(|| async { "ok" }))
            .layer(middleware::from_fn(validate))
    }

    #[tokio::test]
    async fn get_requests_bypass_csrf() {
        let resp = app()
            .oneshot(Request::get("/get").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn post_without_origin_is_forbidden() {
        let resp = app()
            .oneshot(
                Request::post("/test")
                    .header("host", "localhost:3000")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn post_with_matching_origin_succeeds() {
        let resp = app()
            .oneshot(
                Request::post("/test")
                    .header("host", "localhost:3000")
                    .header("origin", "http://localhost:3000")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"ok");
    }

    #[tokio::test]
    async fn post_with_wrong_origin_is_forbidden() {
        let resp = app()
            .oneshot(
                Request::post("/test")
                    .header("host", "localhost:3000")
                    .header("origin", "http://evil.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn post_with_matching_referer_succeeds() {
        let resp = app()
            .oneshot(
                Request::post("/test")
                    .header("host", "localhost:3000")
                    .header("referer", "http://localhost:3000/page")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[test]
    fn origin_matching() {
        assert!(origin_matches_host(
            "http://localhost:3000",
            "localhost:3000"
        ));
        assert!(origin_matches_host("https://example.com", "example.com"));
        assert!(origin_matches_host(
            "http://localhost:3000/some/path",
            "localhost:3000"
        ));
        assert!(!origin_matches_host("http://evil.com", "localhost:3000"));
        assert!(!origin_matches_host(
            "http://localhost:9999",
            "localhost:3000"
        ));
    }
}
