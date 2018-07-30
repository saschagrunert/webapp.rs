#[macro_use]
extern crate lazy_static;
extern crate reqwest;
extern crate serde_cbor;
extern crate toml;
extern crate webapp;
extern crate webapp_backend;

use reqwest::{Client, StatusCode};
use serde_cbor::{from_slice, to_vec};
use std::{fs::read_to_string, sync::Mutex, thread, time::Duration};
use webapp::{
    config::Config,
    protocol::{model::Session, request, response},
    CONFIG_FILENAME,
};
use webapp_backend::Server;

lazy_static! {
    static ref PORT: Mutex<u32> = Mutex::new(30000);
}

fn create_testserver() -> (String, Config) {
    // Create the server thread
    let config_string = read_to_string(format!("../{}", CONFIG_FILENAME)).unwrap();
    let mut config: Config = toml::from_str(&config_string).unwrap();
    let mut port = PORT.lock().unwrap();
    *port += 1;
    config.server.port = port.to_string();
    config.server.tls = false;

    let config_clone = config.clone();
    thread::spawn(move || Server::new(&config_clone).unwrap().start());

    // Wait until the server is up
    thread::sleep(Duration::from_millis(300));

    (format!("http://{}:{}", config.server.ip, config.server.port), config)
}

#[test]
fn succeed_to_login_with_credentials() {
    // Given
    let (url, config) = create_testserver();

    // When
    let request = to_vec(&request::LoginCredentials {
        username: "username".to_owned(),
        password: "username".to_owned(),
    }).unwrap();
    let mut res = Client::new()
        .post(&(url + &config.api.login_credentials))
        .body(request)
        .send()
        .unwrap();
    let mut body = vec![];
    res.copy_to(&mut body).unwrap();
    let response::Login(session) = from_slice(&body).unwrap();

    // Then
    assert!(res.status().is_success());
    assert_eq!(session.token.len(), 211);
}

#[test]
fn fail_to_login_with_wrong_credentials() {
    // Given
    let (url, config) = create_testserver();

    // When
    let request = to_vec(&request::LoginCredentials {
        username: "username".to_owned(),
        password: "password".to_owned(),
    }).unwrap();
    let res = Client::new()
        .post(&(url + &config.api.login_credentials))
        .body(request)
        .send()
        .unwrap();

    // Then
    assert_eq!(res.status(), StatusCode::Unauthorized);
}

#[test]
fn succeed_to_login_with_session() {
    // Given
    let (url, config) = create_testserver();

    // When
    let mut request = to_vec(&request::LoginCredentials {
        username: "username".to_owned(),
        password: "username".to_owned(),
    }).unwrap();
    let mut res = Client::new()
        .post(&(url.clone() + &config.api.login_credentials))
        .body(request)
        .send()
        .unwrap();
    let mut body = vec![];
    res.copy_to(&mut body).unwrap();
    let response::Login(session) = from_slice(&body).unwrap();

    request = to_vec(&request::LoginSession(session)).unwrap();
    res = Client::new()
        .post(&(url + &config.api.login_session))
        .body(request)
        .send()
        .unwrap();
    body.clear();
    res.copy_to(&mut body).unwrap();
    let response::Login(new_session) = from_slice(&body).unwrap();

    // Then
    assert!(res.status().is_success());
    assert_eq!(new_session.token.len(), 211);
}

#[test]
fn fail_to_login_with_wrong_session() {
    // Given
    let (url, config) = create_testserver();

    // When
    let request = to_vec(&request::LoginSession(Session {
        token: "wrong".to_owned(),
    })).unwrap();
    let res = Client::new()
        .post(&(url + &config.api.login_session))
        .body(request)
        .send()
        .unwrap();

    // Then
    assert_eq!(res.status(), StatusCode::Unauthorized);
}

#[test]
fn succeed_to_logout() {
    // Given
    let (url, config) = create_testserver();

    // When
    let request = to_vec(&request::Logout(Session {
        token: "wrong".to_owned(),
    })).unwrap();
    let res = Client::new()
        .post(&(url + &config.api.logout))
        .body(request)
        .send()
        .unwrap();

    // Then
    assert!(res.status().is_success());
}
