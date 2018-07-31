//! Response specific implementations

use protocol::model::Session;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
/// The login response
pub struct Login(pub Session);

#[derive(Debug, PartialEq, Deserialize, Serialize)]
/// The logout response
pub struct Logout;
