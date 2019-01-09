//! Request messages
use crate::protocol::model::Session;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
/// The credentials based login request
pub struct LoginCredentials {
    /// The username
    pub username: String,

    /// The password
    pub password: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
/// The session based login request
pub struct LoginSession(pub Session);

#[derive(Debug, PartialEq, Deserialize, Serialize)]
/// The logout request
pub struct Logout(pub Session);
