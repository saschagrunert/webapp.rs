use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: i64,
    iat: i64,
    jti: String,
}

fn secret() -> Vec<u8> {
    env::var("JWT_SECRET")
        .unwrap_or_else(|_| "change-me-in-production".into())
        .into_bytes()
}

pub fn create_token(username: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let claims = Claims {
        sub: username.to_owned(),
        exp: (now + Duration::hours(1)).timestamp(),
        iat: now.timestamp(),
        jti: uuid::Uuid::new_v4().to_string(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&secret()),
    )
}

pub fn verify_token(token: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(&secret()),
        &Validation::default(),
    )?;
    Ok(data.claims.sub)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_verify_token() {
        let token = create_token("testuser").unwrap();
        let username = verify_token(&token).unwrap();
        assert_eq!(username, "testuser");
    }

    #[test]
    fn verify_invalid_token_fails() {
        assert!(verify_token("invalid-token").is_err());
    }

    #[test]
    fn verify_expired_token_fails() {
        let claims = Claims {
            sub: "testuser".to_owned(),
            exp: (Utc::now() - Duration::hours(1)).timestamp(),
            iat: (Utc::now() - Duration::hours(2)).timestamp(),
            jti: uuid::Uuid::new_v4().to_string(),
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&secret()),
        )
        .unwrap();
        assert!(verify_token(&token).is_err());
    }
}
