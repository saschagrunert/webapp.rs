//! The main protocol handling

#[cfg(feature = "db")]
use schema::sessions;
use serde_cbor::to_vec;

#[derive(Debug, Deserialize, Serialize)]
/// All possible request variants
pub enum Request {
    /// Possible variants of a login request
    Login(Login),

    /// A logout request with a provided session
    Logout(Session),
}

impl Request {
    /// Consume the object into a vector
    pub fn to_vec(&self) -> Option<Vec<u8>> {
        to_vec(self).ok()
    }
}

#[derive(Debug, Deserialize, Serialize)]
/// Possible login request variants
pub enum Login {
    /// A credentials based request
    Credentials {
        /// The username
        username: String,

        /// The password
        password: String,
    },

    /// A session based request
    Session(Session),
}

#[derive(Debug, Deserialize, Serialize)]
/// All possible response variants
pub enum Response {
    /// A generic error from the server, which is not recoverable
    Error,

    /// A login response which returns a session on success
    Login(Result<Session, ResponseError>),

    /// A logout response
    Logout(Result<(), ResponseError>),
}

#[derive(Debug, Fail, Deserialize, Serialize)]
/// All possible response errors
pub enum ResponseError {
    #[fail(display = "wrong username or password")]
    /// Wrong username or password
    WrongUsernamePassword,

    #[fail(display = "unable to create session token")]
    /// Session token creation failed
    CreateToken,

    #[fail(display = "unable to verify session token")]
    /// Session token verification failed
    VerifyToken,

    #[fail(display = "unable to modify database entry")]
    /// Database communication failed
    Database,

    #[fail(display = "unable to insert session into database")]
    /// Session insert in database failed
    InsertSession,

    #[fail(display = "unable to update session within database")]
    /// Session update in database failed
    UpdateSession,

    #[fail(display = "unable to delete session within database")]
    /// Session deletion in database failed
    DeleteSession,
}

#[cfg_attr(feature = "db", derive(Insertable, Queryable))]
#[cfg_attr(feature = "db", table_name = "sessions")]
#[derive(Debug, Deserialize, Serialize)]
/// A session representation
pub struct Session {
    /// The actual session token
    pub token: String,
}
