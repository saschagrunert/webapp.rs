//! Everything related to web token handling

use backend::server::ServerError;
use failure::Error;
use jsonwebtoken::{decode, encode, Header, Validation};
use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};
use time::get_time;
use uuid::Uuid;

lazy_static! {
    static ref SECRET: String = Uuid::new_v4().to_string();
}

const DEFAULT_TOKEN_VALIDITY: i64 = 3600;

/// Generic multi processing ready token storage type
#[derive(Clone, Debug, Default)]
pub struct TokenStore(Arc<RwLock<HashSet<String>>>);

impl TokenStore {
    /// Insert a new token
    pub fn create(&self, username: &str) -> Result<String, Error> {
        let token = Token::create(username, DEFAULT_TOKEN_VALIDITY)?;
        debug!("New token: {}", token);
        self.0
            .try_write()
            .map_err(|_| Error::from(ServerError::CreateToken))?
            .insert(token.clone());
        trace!("All current token: {:?}", self.0);
        Ok(token)
    }

    /// Update old_token for new_token
    pub fn verify(&self, token: &str) -> Result<String, Error> {
        let new_token = Token::verify(token)?;
        debug!("Token {} verified, new token: {}", token, new_token);
        let mut data = self.0.try_write().map_err(|_| Error::from(ServerError::UpdateToken))?;
        data.remove(token);
        data.insert(new_token.clone());
        Ok(new_token)
    }

    /// Remove a token from the storage
    pub fn remove(&self, token: &str) -> Result<(), Error> {
        self.0
            .try_write()
            .map_err(|_| Error::from(ServerError::RemoveToken))?
            .remove(token);
        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
/// A web token
struct Token {
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
    pub fn create(username: &str, validity: i64) -> Result<String, Error> {
        let claim = Token {
            sub: username.to_owned(),
            exp: get_time().sec + validity,
            iat: get_time().sec,
            jti: Uuid::new_v4().to_string(),
        };
        encode(&Header::default(), &claim, SECRET.as_ref()).map_err(|_| ServerError::CreateToken.into())
    }

    /// Verify the validity of a token and get a new one
    pub fn verify(token: &str) -> Result<String, Error> {
        let data = decode::<Token>(token, SECRET.as_ref(), &Validation::default())
            .map_err(|_| Error::from(ServerError::VerifyToken))?;
        Self::create(&data.claims.sub, DEFAULT_TOKEN_VALIDITY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn succeed_to_create_a_token() {
        assert!(Token::create("", 1).is_ok());
    }

    #[test]
    fn succeed_to_verify_a_token() {
        let t = Token::create("", 1).unwrap();
        assert!(Token::verify(&t).is_ok());
    }

    #[test]
    fn fail_to_verify_a_token() {
        let t = Token::create("", -1).unwrap();
        assert!(Token::verify(&t).is_err());
    }
}
