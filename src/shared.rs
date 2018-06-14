//! Shared models for the frontend and backend

pub mod protocol_capnp {
    #![allow(unknown_lints)]
    #![allow(clippy)]
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/src/protocol_capnp.rs"));
}

#[derive(Deserialize, Serialize, Debug)]
/// The most general message abstraction
pub enum WsMessage {
    LoginRequest(LoginRequestData),
    LoginResponse(LoginResponseData),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
/// The data for a login request
pub struct LoginRequestData {
    /// The username
    pub username: String,

    /// The password
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug)]
/// The data for a login response
pub struct LoginResponseData {
    /// Inidicates if the login succeed
    pub success: bool,
}
