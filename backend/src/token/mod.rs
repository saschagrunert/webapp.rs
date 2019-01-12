//! Everything related to web token handling

use actix_web::{HttpResponse, ResponseError};
use failure::Fail;
use jsonwebtoken::{decode, encode, Header, Validation};
use serde_derive::{Deserialize, Serialize};
use time::get_time;
use uuid::Uuid;

mod test;

const SECRET: &[u8] = b"my_secret";

#[derive(Debug, Fail)]
/// Token handling related errors
pub enum TokenError {
    #[fail(display = "unable to create session token")]
    /// Session token creation failed
    Create,

    #[fail(display = "unable to verify session token")]
    /// Session token verification failed
    Verify,
}

impl ResponseError for TokenError {
    fn error_response(&self) -> HttpResponse {
        match self {
            TokenError::Create => HttpResponse::InternalServerError().into(),
            TokenError::Verify => HttpResponse::Unauthorized().into(),
        }
    }
}

#[derive(Deserialize, Serialize)]
/// A web token
pub struct Token {
    /// The subject of the token
    sub: String,

    /// The exipration date of the token
    exp: i64,

    /// The issued at field
    iat: i64,

    /// The token id
    jti: String,
}

impl Token {
    /// Create a new default token for a given username
    pub fn create(username: &str) -> Result<String, TokenError> {
        const DEFAULT_TOKEN_VALIDITY: i64 = 3600;
        let claim = Token {
            sub: username.to_owned(),
            exp: get_time().sec + DEFAULT_TOKEN_VALIDITY,
            iat: get_time().sec,
            jti: Uuid::new_v4().to_string(),
        };
        encode(&Header::default(), &claim, SECRET).map_err(|_| TokenError::Create)
    }

    /// Verify the validity of a token and get a new one
    pub fn verify(token: &str) -> Result<String, TokenError> {
        let data = decode::<Token>(token, SECRET, &Validation::default())
            .map_err(|_| TokenError::Verify)?;
        Self::create(&data.claims.sub)
    }
}
