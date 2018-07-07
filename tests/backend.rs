#[macro_use]
extern crate lazy_static;
extern crate capnp;
extern crate toml;
extern crate tungstenite;
extern crate url;
extern crate webapp;

use capnp::{
    message::{Builder, ReaderOptions},
    serialize_packed::{read_message, write_message},
};
use std::{fs::read_to_string, net::TcpStream, sync::Mutex, thread, time::Duration};
use tungstenite::{connect, Message, WebSocket};
use url::Url;
use webapp::{
    config::Config,
    protocol_capnp::{request, response},
    Server, CONFIG_FILENAME,
};

lazy_static! {
    static ref PORT: Mutex<u32> = Mutex::new(30000);
}

fn create_testserver() -> WebSocket<TcpStream> {
    // Create the server thread
    let config_string = read_to_string(CONFIG_FILENAME).unwrap();
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
    let mut builder = Builder::new_default();
    let mut data = Vec::new();
    {
        let mut creds = builder.init_root::<request::Builder>().init_login().init_credentials();
        creds.set_username("username");
        creds.set_password("username");
    }
    write_message(&mut data, &builder).unwrap();
    socket.write_message(Message::binary(data)).unwrap();

    // Then
    match socket.read_message().unwrap() {
        Message::Binary(b) => {
            let reader = read_message(&mut b.as_ref(), ReaderOptions::new()).unwrap();
            match reader.get_root::<response::Reader>().unwrap().which().unwrap() {
                response::Login(d) => match d.which().unwrap() {
                    response::login::Token(_) => {}
                    _ => panic!("Wrong response content"),
                },
                _ => panic!("Wrong response type"),
            };
        }
        _ => panic!("Wrong message type"),
    }
}

#[test]
fn fail_to_login_with_wrong_username_and_password() {
    // Given
    let mut socket = create_testserver();

    // When
    let mut builder = Builder::new_default();
    let mut data = Vec::new();
    {
        let mut creds = builder.init_root::<request::Builder>().init_login().init_credentials();
        creds.set_username("username");
        creds.set_password("password");
    }
    write_message(&mut data, &builder).unwrap();
    socket.write_message(Message::binary(data)).unwrap();

    // Then
    match socket.read_message().unwrap() {
        Message::Binary(b) => {
            let reader = read_message(&mut b.as_ref(), ReaderOptions::new()).unwrap();
            match reader.get_root::<response::Reader>().unwrap().which().unwrap() {
                response::Login(d) => match d.which().unwrap() {
                    response::login::Error(_) => {}
                    _ => panic!("Wrong response content"),
                },
                _ => panic!("Wrong response type"),
            };
        }
        _ => panic!("Wrong message type"),
    }
}

#[test]
fn succeed_to_login_with_token() {
    // Given
    let mut socket = create_testserver();

    // When
    let mut builder = Builder::new_default();
    let mut data = Vec::new();
    {
        let mut creds = builder.init_root::<request::Builder>().init_login().init_credentials();
        creds.set_username("username");
        creds.set_password("username");
    }
    write_message(&mut data, &builder).unwrap();
    socket.write_message(Message::binary(data)).unwrap();

    // Then
    let token = match socket.read_message().unwrap() {
        Message::Binary(b) => {
            let reader = read_message(&mut b.as_ref(), ReaderOptions::new()).unwrap();
            match reader.get_root::<response::Reader>().unwrap().which().unwrap() {
                response::Login(d) => match d.which().unwrap() {
                    response::login::Token(token) => token.unwrap().to_owned(),
                    _ => panic!("Wrong response content"),
                },
                _ => panic!("Wrong response type"),
            }
        }
        _ => panic!("Wrong message type"),
    };

    // And When
    let mut token_data = Vec::new();
    builder.init_root::<request::Builder>().init_login().set_token(&token);
    write_message(&mut token_data, &builder).unwrap();
    socket.write_message(Message::binary(token_data)).unwrap();

    // Then
    match socket.read_message().unwrap() {
        Message::Binary(b) => {
            let reader = read_message(&mut b.as_ref(), ReaderOptions::new()).unwrap();
            match reader.get_root::<response::Reader>().unwrap().which().unwrap() {
                response::Login(d) => match d.which().unwrap() {
                    response::login::Token(_) => {}
                    _ => panic!("Wrong response content"),
                },
                _ => panic!("Wrong response type"),
            }
        }
        _ => panic!("Wrong message type"),
    }
}

#[test]
fn fail_to_login_with_wrong_token() {
    // Given
    let mut socket = create_testserver();

    // When
    let mut builder = Builder::new_default();
    let mut data = Vec::new();
    let token = "wrong_token".to_owned();
    builder.init_root::<request::Builder>().init_login().set_token(&token);
    write_message(&mut data, &builder).unwrap();
    socket.write_message(Message::binary(data)).unwrap();

    // Then
    match socket.read_message().unwrap() {
        Message::Binary(b) => {
            let reader = read_message(&mut b.as_ref(), ReaderOptions::new()).unwrap();
            match reader.get_root::<response::Reader>().unwrap().which().unwrap() {
                response::Login(d) => match d.which().unwrap() {
                    response::login::Error(_) => {}
                    _ => panic!("Wrong response content"),
                },
                _ => panic!("Wrong response type"),
            };
        }
        _ => panic!("Wrong message type"),
    }
}

#[test]
fn succeed_to_logout() {
    // Given
    let mut socket = create_testserver();

    // When
    let mut builder = Builder::new_default();
    let mut data = Vec::new();
    let token = "token".to_owned();
    builder.init_root::<request::Builder>().set_logout(&token);
    write_message(&mut data, &builder).unwrap();
    socket.write_message(Message::binary(data)).unwrap();

    // Then
    match socket.read_message().unwrap() {
        Message::Binary(b) => {
            let reader = read_message(&mut b.as_ref(), ReaderOptions::new()).unwrap();
            match reader.get_root::<response::Reader>().unwrap().which().unwrap() {
                response::Logout(_) => {}
                _ => panic!("Wrong response type"),
            };
        }
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
