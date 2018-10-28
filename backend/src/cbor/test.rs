//! The cbor tests

#![cfg(test)]

use actix_web::{test::TestRequest, HttpRequest, HttpResponse};
use cbor::{CborRequest, CborResponseBuilder};
use failure::{Fail, Fallible};
use futures::Future;
use serde_cbor::to_vec;
use webapp::protocol::{request, response};

fn build_request() -> TestRequest<()> {
    TestRequest::with_header("content-type", "application/cbor")
}

#[test]
fn succeed_to_decode_request() -> Fallible<()> {
    // Given
    let login = request::LoginCredentials {
        username: "username".to_owned(),
        password: "password".to_owned(),
    };

    // When
    let payload = to_vec(&login)?;
    let request: HttpRequest<()> = build_request().set_payload(payload).finish();
    let result: request::LoginCredentials = CborRequest::new(&request).wait()?;

    // Then
    assert_eq!(result, login);
    Ok(())
}

#[test]
fn fail_to_decode_empty_request() -> Fallible<()> {
    // Given
    let request: HttpRequest<()> = build_request().finish();

    // When
    let result: Result<(), _> = CborRequest::new(&request).wait();

    // When
    assert!(&result
        .unwrap_err()
        .cause()
        .unwrap()
        .to_string()
        .contains("EOF"));
    Ok(())
}

#[test]
fn fail_to_decode_wrong_request() -> Fallible<()> {
    // Given
    let payload: Vec<u8> = (1..10).collect();

    // When
    let request: HttpRequest<()> = build_request().set_payload(payload).finish();
    let result: Result<(), _> = CborRequest::new(&request).wait();

    // Then
    assert!(&result
        .unwrap_err()
        .cause()
        .unwrap()
        .to_string()
        .contains("invalid type"));
    Ok(())
}

#[test]
fn fail_to_decode_wrong_typed_request() -> Fallible<()> {
    // Given
    let login = request::LoginCredentials {
        username: "username".to_owned(),
        password: "password".to_owned(),
    };

    // When
    let payload = to_vec(&login)?;
    let request: HttpRequest<()> = build_request().set_payload(payload).finish();
    let result: Result<request::Logout, _> = CborRequest::new(&request).wait();

    // Then
    assert!(&result
        .unwrap_err()
        .cause()
        .unwrap()
        .to_string()
        .contains("missing field"));
    Ok(())
}

#[test]
fn succeed_to_encode_response() {
    // Given
    // When
    let response = HttpResponse::Ok().cbor(response::Logout);

    // Then
    assert!(response.is_ok());
}
