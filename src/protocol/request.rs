//! Request messages

use protocol::model::Session;
use serde_cbor::to_vec;

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
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
