//! String representations for the user interface

macro_rules! strings {
    ($(($name:ident, $content:expr)),*) => (
        $(pub static $name: &str = $content;)*
    )
}

strings!(
    (SERVER_COMMUNICATION_CLOSED, "Server connection closed"),
    (ERROR_SERVER_INTERNAL, "Internal server error"),
    (ERROR_SERVER_COMMUNICATION, "Server communication unavailable or broken")
);
