#![feature(test)]

#[macro_use]
extern crate lazy_static;
extern crate reqwest;
extern crate serde_cbor;
extern crate test;
extern crate url;
extern crate webapp;
extern crate webapp_backend;

use reqwest::Client;
use serde_cbor::to_vec;
use std::{sync::Mutex, thread};
use test::Bencher;
use url::Url;
use webapp::{config::Config, protocol::request, API_URL_LOGIN_CREDENTIALS, CONFIG_FILENAME};
use webapp_backend::Server;

lazy_static! {
    static ref PORT: Mutex<u16> = Mutex::new(31000);
}

fn get_next_port() -> u16 {
    let mut port = PORT.lock().unwrap();
    *port += 1;
    *port
}

pub fn create_testserver() -> Url {
    // Prepare the configuration
    let mut config = Config::new(&format!("../{}", CONFIG_FILENAME)).unwrap();

    // Set the test configuration
    let mut url = Url::parse(&config.server.url).unwrap();
    url.set_port(Some(get_next_port())).unwrap();
    config.server.url = url.to_string();
    config.server.redirect_from = vec![];

    // Start the server
    let config_clone = config.clone();
    thread::spawn(move || Server::new(&config_clone).unwrap().start());

    // Wait until the server is up
    loop {
        if let Ok(res) = Client::new().get(url.clone()).send() {
            if res.status().is_success() {
                break;
            }
        }
    }

    // Return the server url
    url
}

#[bench]
fn bench_login_credentials_low(b: &mut Bencher) {
    // Given
    let mut url = create_testserver();
    url.set_path(API_URL_LOGIN_CREDENTIALS);

    // When
    let len = (0..10).map(|_| "X").collect::<String>();
    let request = to_vec(&request::LoginCredentials {
        username: len.to_owned(),
        password: len,
    }).unwrap();
    println!("Request len: {}", request.len());

    // Then
    b.iter(|| {
        assert!(
            Client::new()
                .post(url.clone())
                .body(request.clone())
                .send()
                .unwrap()
                .status()
                .is_success()
        );
    });
}

#[bench]
fn bench_login_credentials_mid(b: &mut Bencher) {
    // Given
    let mut url = create_testserver();
    url.set_path(API_URL_LOGIN_CREDENTIALS);

    // When
    let len = (0..200).map(|_| "X").collect::<String>();
    let request = to_vec(&request::LoginCredentials {
        username: len.to_owned(),
        password: len,
    }).unwrap();
    println!("Request len: {}", request.len());

    // Then
    b.iter(|| {
        assert!(
            Client::new()
                .post(url.clone())
                .body(request.clone())
                .send()
                .unwrap()
                .status()
                .is_success()
        );
    });
}

#[bench]
fn bench_login_credentials_high(b: &mut Bencher) {
    // Given
    let mut url = create_testserver();
    url.set_path(API_URL_LOGIN_CREDENTIALS);

    // When
    let len = (0..1000).map(|_| "X").collect::<String>();
    let request = to_vec(&request::LoginCredentials {
        username: len.to_owned(),
        password: len,
    }).unwrap();
    println!("Request len: {}", request.len());

    // Then
    b.iter(|| {
        assert!(
            Client::new()
                .post(url.clone())
                .body(request.clone())
                .send()
                .unwrap()
                .status()
                .is_success()
        );
    });
}
