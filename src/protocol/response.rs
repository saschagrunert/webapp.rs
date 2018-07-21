//! Response messages

use protocol::model::Session;

#[derive(Clone, Debug, Deserialize, Serialize)]
/// All possible response variants
pub enum Response {
    /// A generic error from the server, which is not recoverable
    Error,

    /// A login response
    Login(Login),

    /// A logout response
    Logout(Result<(), ResponseError>),
}

#[derive(Clone, Debug, Fail, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
/// Possible login request variants
pub enum Login {
    /// A login response for given credentials which returns a session on success
    Credentials(Result<Session, ResponseError>),

    /// A login response for a given session which returns a session on success
    Session(Result<Session, ResponseError>),
}
