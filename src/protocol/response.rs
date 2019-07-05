//! Response specific implementations
use crate::protocol::model::Session;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
/// The login response
pub struct Login(pub Session);

#[derive(Debug, PartialEq, Deserialize, Serialize)]
/// The logout response
pub struct Logout;
