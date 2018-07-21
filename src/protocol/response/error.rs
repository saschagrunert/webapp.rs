//! Response messages

#[cfg(feature = "backend")]
use actix::MailboxError;
use std::convert::From;

macro_rules! from_impl {
    ($from:path, $for:ident, $variant:ident) => {
        impl From<$from> for $for {
            fn from(e: $from) -> Self {
                $for::$variant(e)
            }
        }
    };
    ($from:path, $for:ident, $to:expr) => {
        impl From<$from> for $for {
            fn from(_: $from) -> Self {
                $to
            }
        }
    };
}

#[derive(Clone, Debug, Fail, Deserialize, Serialize)]
/// Database related errors
pub enum DatabaseError {
    #[fail(display = "unable communicate to database")]
    /// Database communication failed
    Communication,

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

#[derive(Clone, Debug, Fail, Deserialize, Serialize)]
/// Token handling related errors
pub enum TokenError {
    #[fail(display = "unable to create session token")]
    /// Session token creation failed
    Create,

    #[fail(display = "unable to verify session token")]
    /// Session token verification failed
    Verify,
}

#[derive(Clone, Debug, Fail, Deserialize, Serialize)]
/// All possible response errors
pub enum LogoutError {
    #[fail(display = "database error")]
    /// Database error occured
    Database(DatabaseError),
}

from_impl!(DatabaseError, LogoutError, Database);
#[cfg(feature = "backend")]
from_impl!(
    MailboxError,
    LogoutError,
    LogoutError::Database(DatabaseError::Communication)
);

#[derive(Clone, Debug, Fail, Deserialize, Serialize)]
/// All possible response errors
pub enum LoginCredentialsError {
    #[fail(display = "wrong username or password")]
    /// Wrong username or password
    WrongUsernamePassword,

    #[fail(display = "database error")]
    /// Database error occured
    Database(DatabaseError),

    #[fail(display = "token error")]
    /// Token error occured
    Token(TokenError),
}

from_impl!(DatabaseError, LoginCredentialsError, Database);
from_impl!(TokenError, LoginCredentialsError, Token);
#[cfg(feature = "backend")]
from_impl!(
    MailboxError,
    LoginCredentialsError,
    LoginCredentialsError::Database(DatabaseError::Communication)
);

#[derive(Clone, Debug, Fail, Deserialize, Serialize)]
/// All possible response errors
pub enum LoginSessionError {
    #[fail(display = "database error")]
    /// Database error occured
    Database(DatabaseError),

    #[fail(display = "token error")]
    /// Token error occured
    Token(TokenError),
}

from_impl!(DatabaseError, LoginSessionError, Database);
from_impl!(TokenError, LoginSessionError, Token);
#[cfg(feature = "backend")]
from_impl!(
    MailboxError,
    LoginSessionError,
    LoginSessionError::Database(DatabaseError::Communication)
);
