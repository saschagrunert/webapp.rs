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

    #[test]
    fn matching_origins() {
        assert!(origin_matches_host(
            "http://localhost:3000",
            "localhost:3000"
        ));
        assert!(origin_matches_host("https://example.com", "example.com"));
        assert!(origin_matches_host(
            "http://localhost:3000/some/path",
            "localhost:3000"
        ));
    }

    #[test]
    fn mismatched_origins() {
        assert!(!origin_matches_host("http://evil.com", "localhost:3000"));
        assert!(!origin_matches_host(
            "http://localhost:9999",
            "localhost:3000"
        ));
    }
}
