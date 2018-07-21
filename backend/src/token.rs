//! Everything related to web token handling

use jsonwebtoken::{decode, encode, Header, Validation};
use time::get_time;
use uuid::Uuid;
use webapp::protocol::TokenError;

const SECRET: &[u8] = b"my_secret";

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
    /// Create a new default token for a given username and a validity in seconds
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
        let data = decode::<Token>(token, SECRET, &Validation::default()).map_err(|_| TokenError::Verify)?;
        Self::create(&data.claims.sub)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn succeed_to_create_a_token() {
        assert!(Token::create("").is_ok());
    }

    #[test]
    fn succeed_to_verify_a_token() {
        let sut = Token::create("").unwrap();
        assert!(Token::verify(&sut).is_ok());
    }
}
