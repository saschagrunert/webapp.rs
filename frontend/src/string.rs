//! String representations for the user interface

macro_rules! strings {
    ($(($name:ident, $content:expr)),*) => (
        $(pub static $name: &str = $content;)*
    )
}

strings!(
    (ERROR_AUTHENTICATION_FAILED, "Authentication failed"),
    (ERROR_SERVER_COMMUNICATION, "Server communication unavailable or broken"),
    (ERROR_SERVER_INTERNAL, "Internal server error"),
    (INPUT_PASSWORD, "Password"),
    (INPUT_USERNAME, "Username"),
    (SERVER_COMMUNICATION_CLOSED, "Server connection closed"),
    (TEXT_CONTENT, "Content"),
    (TEXT_LOGIN, "Login"),
    (TEXT_LOGOUT, "Logout"),
    (TEXT_REGISTER, "Register")
);
