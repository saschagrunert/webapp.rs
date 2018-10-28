#![feature(test)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate failure;
extern crate reqwest;
extern crate serde_cbor;
extern crate test;
extern crate url;
extern crate webapp;
extern crate webapp_backend;

use failure::Fallible;
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

pub fn create_testserver() -> Fallible<Url> {
    // Prepare the configuration
    let mut config = Config::from_file(&format!("../{}", CONFIG_FILENAME))?;

    // Set the test configuration
    let mut url = Url::parse(&config.server.url)?;
    url.set_port(Some(get_next_port()))
        .map_err(|_| format_err!("Unable to set server port"))?;
    config.server.url = url.to_string();
    config.server.redirect_from = vec![];

    // Start the server
    let config_clone = config.clone();
    thread::spawn(move || Server::from_config(&config_clone).unwrap().start());

    // Wait until the server is up
    loop {
        if let Ok(res) = Client::new().get(url.clone()).send() {
            if res.status().is_success() {
                break;
            }
        }
    }

    // Return the server url
    Ok(url)
}

#[bench]
fn bench_login_credentials_low(b: &mut Bencher) -> Fallible<()> {
    // Given
    let mut url = create_testserver()?;
    url.set_path(API_URL_LOGIN_CREDENTIALS);

    // When
    let len = (0..10).map(|_| "X").collect::<String>();
    let request = to_vec(&request::LoginCredentials {
        username: len.to_owned(),
        password: len,
    })?;
    println!("Request len: {}", request.len());

    // Then
    b.iter(|| {
        assert!(Client::new()
            .post(url.clone())
            .body(request.clone())
            .send()
            .unwrap()
            .status()
            .is_success());
    });
    Ok(())
}

#[bench]
fn bench_login_credentials_mid(b: &mut Bencher) -> Fallible<()> {
    // Given
    let mut url = create_testserver()?;
    url.set_path(API_URL_LOGIN_CREDENTIALS);

    // When
    let len = (0..200).map(|_| "X").collect::<String>();
    let request = to_vec(&request::LoginCredentials {
        username: len.to_owned(),
        password: len,
    })?;
    println!("Request len: {}", request.len());

    // Then
    b.iter(|| {
        assert!(Client::new()
            .post(url.clone())
            .body(request.clone())
            .send()
            .unwrap()
            .status()
            .is_success());
    });
    Ok(())
}

#[bench]
fn bench_login_credentials_high(b: &mut Bencher) -> Fallible<()> {
    // Given
    let mut url = create_testserver()?;
    url.set_path(API_URL_LOGIN_CREDENTIALS);

    // When
    let len = (0..1000).map(|_| "X").collect::<String>();
    let request = to_vec(&request::LoginCredentials {
        username: len.to_owned(),
        password: len,
    })?;
    println!("Request len: {}", request.len());

    // Then
    b.iter(|| {
        assert!(Client::new()
            .post(url.clone())
            .body(request.clone())
            .send()
            .unwrap()
            .status()
            .is_success());
    });
    Ok(())
}
