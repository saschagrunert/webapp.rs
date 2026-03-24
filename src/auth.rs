use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
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

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    let parsed = PasswordHash::new(hash).map_err(|e| e.to_string())?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
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

pub fn token_expiry() -> chrono::DateTime<Utc> {
    Utc::now() + Duration::hours(1)
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

    #[test]
    fn hash_and_verify_password() {
        let hash = hash_password("my-secret").unwrap();
        assert!(verify_password("my-secret", &hash).unwrap());
        assert!(!verify_password("wrong-password", &hash).unwrap());
    }

    #[test]
    fn hash_produces_unique_salts() {
        let h1 = hash_password("same").unwrap();
        let h2 = hash_password("same").unwrap();
        assert_ne!(h1, h2);
    }
}
