#![feature(test)]

extern crate reqwest;
extern crate serde_cbor;
extern crate test;
extern crate url;
extern crate webapp;
extern crate webapp_backend;

use reqwest::Client;
use serde_cbor::to_vec;
use std::thread;
use test::Bencher;
use url::Url;
use webapp::{config::Config, protocol::request, API_URL_LOGIN_CREDENTIALS, CONFIG_FILENAME};
use webapp_backend::Server;

pub fn create_testserver() -> Url {
    // Prepare the configuration
    let mut config = Config::new(&format!("../{}", CONFIG_FILENAME)).unwrap();

    // Set the test configuration
    let url = Url::parse(&config.server.url).unwrap();
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
fn bench_succeed_login_credentials_49bytes(b: &mut Bencher) {
    // Given
    let mut url = create_testserver();
    url.set_path(API_URL_LOGIN_CREDENTIALS);

    // When
    let request = to_vec(&request::LoginCredentials {
        username: "username_12345".to_owned(),
        password: "username_12345".to_owned(),
    }).unwrap();

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
