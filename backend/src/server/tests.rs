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
fn fail_to_create_a_server_with_wrong_addr() {
    let mut config = get_config();
    config.server.ip = "".to_owned();
    assert!(Server::new(&config).is_err());
}

#[test]
fn fail_to_create_a_server_with_wrong_port() {
    let mut config = get_config();
    config.server.port = "-1".to_owned();
    assert!(Server::new(&config).is_err());
}
