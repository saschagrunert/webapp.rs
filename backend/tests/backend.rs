#[macro_use]
extern crate lazy_static;
extern crate serde_cbor;
extern crate toml;
extern crate tungstenite;
extern crate url;
extern crate webapp;
extern crate webapp_backend;

use serde_cbor::from_slice;
use std::{fs::read_to_string, net::TcpStream, sync::Mutex, thread, time::Duration};
use tungstenite::{connect, Message, WebSocket};
use url::Url;
use webapp::CONFIG_FILENAME;
use webapp::{
    config::Config,
    protocol::{self, request, response, Response, Session},
};
use webapp_backend::Server;

lazy_static! {
    static ref PORT: Mutex<u32> = Mutex::new(30000);
}

fn create_testserver() -> WebSocket<TcpStream> {
    // Create the server thread
    let config_string = read_to_string(format!("../{}", CONFIG_FILENAME)).unwrap();
    let mut config: Config = toml::from_str(&config_string).unwrap();
    let mut port = PORT.lock().unwrap();
    *port += 1;
    config.server.port = port.to_string();
    config.server.tls = false;
    let ws_url = format!("ws://{}:{}/ws", config.server.ip, config.server.port);
    thread::spawn(move || Server::new(&config).unwrap().start());

    // Wait until the server is up
    thread::sleep(Duration::from_millis(300));

    // Connect to the websocket
    let (socket, _) = connect(Url::parse(&ws_url).unwrap()).unwrap();
    socket
}

#[test]
fn succeed_to_login_with_username_and_password() {
    // Given
    let mut socket = create_testserver();

    // When
    let data = protocol::Request::Login(request::Login::Credentials {
        username: "username".to_owned(),
        password: "username".to_owned(),
    }).to_vec()
        .unwrap();
    socket.write_message(Message::binary(data)).unwrap();

    // Then
    match socket.read_message().unwrap() {
        Message::Binary(b) => match from_slice(&b) {
            Ok(Response::Login(response::Login::Credentials(Ok(_)))) => {}
            _ => panic!("Wrong response type"),
        },
        _ => panic!("Wrong message type"),
    }
}

#[test]
fn fail_to_login_with_wrong_username_and_password() {
    // Given
    let mut socket = create_testserver();

    // When
    let data = protocol::Request::Login(request::Login::Credentials {
        username: "username".to_owned(),
        password: "password".to_owned(),
    }).to_vec()
        .unwrap();
    socket.write_message(Message::binary(data)).unwrap();

    // Then
    match socket.read_message().unwrap() {
        Message::Binary(b) => match from_slice(&b) {
            Ok(Response::Login(response::Login::Credentials(Err(_)))) => {}
            _ => panic!("Wrong response type"),
        },
        _ => panic!("Wrong message type"),
    }
}

#[test]
fn succeed_to_login_with_session() {
    // Given
    let mut socket = create_testserver();

    // When
    let data = protocol::Request::Login(request::Login::Credentials {
        username: "username".to_owned(),
        password: "username".to_owned(),
    }).to_vec()
        .unwrap();
    socket.write_message(Message::binary(data)).unwrap();

    // Then
    let session = match socket.read_message().unwrap() {
        Message::Binary(b) => match from_slice(&b) {
            Ok(Response::Login(response::Login::Credentials(Ok(session)))) => session,
            _ => panic!("Wrong response type"),
        },
        _ => panic!("Wrong message type"),
    };

    // And When
    let session_data = protocol::Request::Login(request::Login::Session(session))
        .to_vec()
        .unwrap();
    socket.write_message(Message::binary(session_data)).unwrap();

    // Then
    match socket.read_message().unwrap() {
        Message::Binary(b) => match from_slice(&b) {
            Ok(Response::Login(response::Login::Session(Ok(_)))) => {}
            _ => panic!("Wrong response type"),
        },
        _ => panic!("Wrong message type"),
    }
}

#[test]
fn fail_to_login_with_wrong_session() {
    // Given
    let mut socket = create_testserver();

    // When
    let data = protocol::Request::Login(request::Login::Session(Session {
        token: "wrong_token".to_owned(),
    })).to_vec()
        .unwrap();
    socket.write_message(Message::binary(data)).unwrap();

    // Then
    match socket.read_message().unwrap() {
        Message::Binary(b) => match from_slice(&b) {
            Ok(Response::Login(response::Login::Session(Err(_)))) => {}
            _ => panic!("Wrong response type"),
        },
        _ => panic!("Wrong message type"),
    }
}

#[test]
fn succeed_to_logout() {
    // Given
    let mut socket = create_testserver();

    // When
    let data = protocol::Request::Logout(Session {
        token: "token".to_owned(),
    }).to_vec()
        .unwrap();
    socket.write_message(Message::binary(data)).unwrap();

    // Then
    match socket.read_message().unwrap() {
        Message::Binary(b) => match from_slice(&b) {
            Ok(Response::Logout(Ok(_))) => {}
            _ => panic!("Wrong response type"),
        },
        _ => panic!("Wrong message type"),
    }
}

#[test]
fn succeed_to_close_the_connection() {
    // Given
    let mut socket = create_testserver();

    // When, Then
    assert!(socket.close(None).is_ok());
}
