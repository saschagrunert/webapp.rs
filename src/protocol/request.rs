//! Request messages
use crate::protocol::model::Session;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
/// The credentials based login request
pub struct LoginCredentials {
    /// The username to login
    pub username: String,

    /// The password to login
    pub password: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
/// The session based login request
pub struct LoginSession(pub Session);

#[derive(Debug, PartialEq, Deserialize, Serialize)]
/// The logout request
pub struct Logout(pub Session);
