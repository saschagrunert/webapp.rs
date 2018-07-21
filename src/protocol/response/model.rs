//! Response messages

use protocol::model::Session;
use protocol::response::error::{LoginCredentialsError, LoginSessionError, LogoutError};

#[derive(Clone, Debug, Deserialize, Serialize)]
/// All possible response variants
pub enum Response {
    /// A generic error from the server, which is not recoverable
    Error,

    /// A login response
    Login(Login),

    /// A logout response
    Logout(Result<(), LogoutError>),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// Possible login request variants
pub enum Login {
    /// A login response for given credentials which returns a session on success
    Credentials(Result<Session, LoginCredentialsError>),

    /// A login response for a given session which returns a session on success
    Session(Result<Session, LoginSessionError>),
}
