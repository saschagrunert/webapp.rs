//! The Server tests

#![cfg(test)]

extern crate toml;

use server::Server;
use std::fs::read_to_string;
use webapp::{config::Config, CONFIG_FILENAME};

fn get_config() -> Config {
    toml::from_str(&read_to_string(format!("../{}", CONFIG_FILENAME)).unwrap()).unwrap()
}

#[test]
fn succeed_to_create_a_server() {
    assert!(Server::new(&get_config()).is_ok());
}

#[test]
fn fail_to_create_a_server_with_wrong_url() {
    let mut config = get_config();
    config.server.url = "".to_owned();
    assert!(Server::new(&config).is_err());
}

#[test]
fn succeed_to_create_a_server_with_tls() {
    let mut config = get_config();
    config.server.url = "https://localhost:30081".to_owned();
    assert!(Server::new(&config).is_ok());
}

#[test]
fn fail_to_create_a_server_with_tls_if_not_found() {
    let mut config = get_config();
    config.server.url = "https://localhost:30082".to_owned();
    config.server.cert = "".to_owned();
    config.server.key = "".to_owned();
    assert!(Server::new(&config).is_err());
}
