//! Request messages

use protocol::model::Session;
use serde_cbor::ser::to_vec_packed;

#[derive(Clone, Debug, Deserialize, Serialize)]
/// All possible request variants
pub enum Request {
    /// Possible variants of a login request
    Login(Login),

    /// A logout request with a provided session
    Logout(Session),
}

impl Request {
    /// Convert the request into a vector of bytes on success
    pub fn to_vec(&self) -> Option<Vec<u8>> {
        to_vec_packed(self).ok()
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
