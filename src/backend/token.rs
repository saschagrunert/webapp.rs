//! Everything related to web token handling

use backend::server::ServerError;
use failure::Error;
use jsonwebtoken::{decode, encode, Header, Validation};
use time::get_time;
use uuid::Uuid;

lazy_static! {
    static ref SECRET: String = Uuid::new_v4().to_string();
}

#[derive(Debug, Deserialize, Serialize)]
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
        Self::create(&data.claims.sub, 3600)
    }
}
