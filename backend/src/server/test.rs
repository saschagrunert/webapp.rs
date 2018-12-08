//! The Server tests

#![cfg(test)]

use crate::server::Server;
use failure::Fallible;
use std::path::PathBuf;
use webapp::{config::Config, CONFIG_FILENAME};

fn get_config() -> Fallible<Config> {
    Ok(Config::from_file(&format!("../{}", CONFIG_FILENAME))?)
}

#[test]
fn succeed_to_create_a_server() -> Fallible<()> {
    // Given
    // When
    // Then
    assert!(Server::from_config(&get_config()?).is_ok());
    Ok(())
}

#[test]
fn fail_to_create_a_server_with_wrong_url() -> Fallible<()> {
    // Given
    let mut config = get_config()?;
    config.server.url = "".to_owned();

    // When
    // Then
    assert!(Server::from_config(&config).is_err());
    Ok(())
}

#[test]
fn succeed_to_create_a_server_with_tls() -> Fallible<()> {
    // Given
    let mut config = get_config()?;
    config.server.url = "https://localhost:30081".to_owned();

    // When
    // Then
    assert!(Server::from_config(&config).is_ok());
    Ok(())
}

#[test]
fn fail_to_create_a_server_with_tls_if_not_found() -> Fallible<()> {
    // Given
    let mut config = get_config()?;
    config.server.url = "https://localhost:30082".to_owned();
    config.server.cert = PathBuf::new();
    config.server.key = PathBuf::new();

    // When
    // Then
    assert!(Server::from_config(&config).is_err());
    Ok(())
}
