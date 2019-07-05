//! Basic models
#[cfg(feature = "backend")]
use crate::schema::sessions;
use serde::{Deserialize, Serialize};
use std::convert::From;

#[cfg_attr(feature = "backend", derive(Insertable, Queryable))]
#[cfg_attr(feature = "backend", table_name = "sessions")]
#[derive(Debug, PartialEq, Deserialize, Serialize)]
/// A session representation
pub struct Session {
    /// The actual session token
    pub token: String,
}

impl Session {
    /// Create a new session from a given token
    pub fn new<T>(token: T) -> Self
    where
        String: From<T>,
    {
        Self {
            token: token.into(),
        }
    }
}
