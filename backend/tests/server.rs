#[macro_use]
extern crate lazy_static;
extern crate reqwest;
extern crate serde_cbor;
extern crate toml;
extern crate url;
extern crate webapp;
extern crate webapp_backend;

use reqwest::{Client, StatusCode};
use serde_cbor::{from_slice, to_vec};
use std::{fs::read_to_string, sync::Mutex, thread, time::Duration};
use url::Url;
use webapp::{
    config::Config,
    protocol::{model::Session, request, response},
    API_URL_LOGIN_CREDENTIALS, API_URL_LOGIN_SESSION, API_URL_LOGOUT, CONFIG_FILENAME,
};
use webapp_backend::Server;

lazy_static! {
    static ref PORT: Mutex<u16> = Mutex::new(30000);
}

fn parse_config() -> Config {
    let config_string = read_to_string(format!("../{}", CONFIG_FILENAME)).unwrap();
    toml::from_str(&config_string).unwrap()
}

fn get_next_port() -> u16 {
    let mut port = PORT.lock().unwrap();
    *port += 1;
    *port
}

fn create_testserver() -> Url {
    // Prepare the configuration
    let mut config = parse_config();

    // Set the test configuration
    let mut url = Url::parse(&config.server.url).unwrap();
    url.set_port(Some(get_next_port())).unwrap();
    config.server.url = url.as_str().to_owned();
    config.server.redirect_from = vec![];

    // Start the server
    let config_clone = config.clone();
    thread::spawn(move || Server::new(&config_clone).unwrap().start());

    // Wait until the server is up
    thread::sleep(Duration::from_millis(300));

    // Return the server url
    url
}

#[test]
fn succeed_to_create_server_with_common_redirects() {
    // Given
    let mut config = parse_config();
    let mut url = Url::parse(&config.server.url).unwrap();
    url.set_port(Some(get_next_port())).unwrap();
    config.server.url = url.as_str().to_owned();

    let redirect_url = "http://127.0.0.1:30666".to_owned();
    config.server.redirect_from = vec![
        redirect_url.clone(),
        "https://localhost:30667".to_owned(),
        "invalid".to_owned(),
    ];

    // When
    let config_clone = config.clone();
    thread::spawn(move || Server::new(&config_clone).unwrap().start());
    thread::sleep(Duration::from_millis(300));
    let res = Client::new().get(&redirect_url).send().unwrap();

    // Then
    assert_eq!(res.url().as_str().contains(&redirect_url), false);
}

#[test]
fn succeed_to_login_with_credentials() {
    // Given
    let mut url = create_testserver();
    url.set_path(API_URL_LOGIN_CREDENTIALS);

    // When
    let request = to_vec(&request::LoginCredentials {
        username: "username".to_owned(),
        password: "username".to_owned(),
    }).unwrap();
    println!("::: url:: {}", url);
    let mut res = Client::new().post(url).body(request).send().unwrap();
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
    let mut url = create_testserver();
    url.set_path(API_URL_LOGIN_CREDENTIALS);

    // When
    let request = to_vec(&request::LoginCredentials {
        username: "username".to_owned(),
        password: "password".to_owned(),
    }).unwrap();
    let res = Client::new().post(url).body(request).send().unwrap();

    // Then
    assert_eq!(res.status(), StatusCode::Unauthorized);
}

#[test]
fn succeed_to_login_with_session() {
    // Given
    let mut url = create_testserver();
    url.set_path(API_URL_LOGIN_CREDENTIALS);

    // When
    let mut request = to_vec(&request::LoginCredentials {
        username: "username".to_owned(),
        password: "username".to_owned(),
    }).unwrap();
    let mut res = Client::new()
        .post(url.clone())
        .body(request)
        .send()
        .unwrap();
    let mut body = vec![];
    res.copy_to(&mut body).unwrap();
    let response::Login(session) = from_slice(&body).unwrap();

    request = to_vec(&request::LoginSession(session)).unwrap();
    url.set_path(API_URL_LOGIN_SESSION);
    res = Client::new().post(url).body(request).send().unwrap();
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
    let mut url = create_testserver();
    url.set_path(API_URL_LOGIN_SESSION);

    // When
    let request = to_vec(&request::LoginSession(Session {
        token: "wrong".to_owned(),
    })).unwrap();
    let res = Client::new().post(url).body(request).send().unwrap();

    // Then
    assert_eq!(res.status(), StatusCode::Unauthorized);
}

#[test]
fn succeed_to_logout() {
    // Given
    let mut url = create_testserver();
    url.set_path(API_URL_LOGOUT);

    // When
    let request = to_vec(&request::Logout(Session {
        token: "wrong".to_owned(),
    })).unwrap();
    let res = Client::new().post(url).body(request).send().unwrap();

    // Then
    assert!(res.status().is_success());
}
