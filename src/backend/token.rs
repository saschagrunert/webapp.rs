//! Everything related to web token handling

use backend::server::ServerError;
use failure::Error;
use jsonwebtoken::{decode, encode, Header, Validation};
use time::get_time;

#[derive(Debug, Deserialize, Serialize)]
/// A web token claim
pub struct Claim {
    /// The subject of the token
    pub sub: String,

    /// The exipration date of the token
    pub exp: i64,
}

impl Claim {
    /// Create a new default token for a given username and a validity in seconds
    pub fn create_token(username: &str, validity: i64) -> Result<String, Error> {
        let claim = Claim {
            sub: username.to_owned(),
            exp: get_time().sec + validity,
        };
        encode(&Header::default(), &claim, b"secret").map_err(|_| ServerError::CreateToken.into())
    }

    /// Verify the validity of a token and get a new one
    pub fn verify_token(token: &str) -> Result<String, Error> {
        let data = decode::<Claim>(token, b"secret", &Validation::default())
            .map_err(|_| Error::from(ServerError::VerifyToken))?;
        Self::create_token(&data.claims.sub, 3600)
    }
}
