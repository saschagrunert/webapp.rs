//! Response specific implementations

use protocol::model::Session;

#[derive(Deserialize, Serialize)]
/// The login response
pub struct Login(pub Session);

#[derive(Deserialize, Serialize)]
/// The logout response
pub struct Logout;
