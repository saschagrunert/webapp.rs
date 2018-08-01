//! The cbor tests

#![cfg(test)]

use actix_web::{test::TestRequest, HttpRequest, HttpResponse};
use cbor::{CborRequest, CborResponseBuilder};
use failure::Fail;
use futures::Future;
use serde_cbor::to_vec;
use webapp::protocol::{request, response};

fn build_request() -> TestRequest<()> {
    TestRequest::with_header("content-type", "application/cbor")
}

#[test]
fn succeed_to_decode_request() {
    let login = request::LoginCredentials {
        username: "username".to_owned(),
        password: "password".to_owned(),
    };
    let payload = to_vec(&login).unwrap();
    let request: HttpRequest<()> = build_request().set_payload(payload).finish();
    let result: request::LoginCredentials = CborRequest::new(&request).wait().unwrap();
    assert_eq!(result, login);
}

#[test]
fn fail_to_decode_empty_request() {
    let request: HttpRequest<()> = build_request().finish();
    let result: Result<(), _> = CborRequest::new(&request).wait();
    assert!(
        &result
            .unwrap_err()
            .cause()
            .unwrap()
            .to_string()
            .contains("EOF")
    );
}

#[test]
fn fail_to_decode_wrong_request() {
    let payload: Vec<u8> = (1..10).collect();
    let request: HttpRequest<()> = build_request().set_payload(payload).finish();
    let result: Result<(), _> = CborRequest::new(&request).wait();
    assert!(
        &result
            .unwrap_err()
            .cause()
            .unwrap()
            .to_string()
            .contains("invalid type")
    );
}

#[test]
fn fail_to_decode_wrong_typed_request() {
    let login = request::LoginCredentials {
        username: "username".to_owned(),
        password: "password".to_owned(),
    };
    let payload = to_vec(&login).unwrap();
    let request: HttpRequest<()> = build_request().set_payload(payload).finish();
    let result: Result<request::Logout, _> = CborRequest::new(&request).wait();
    assert!(
        &result
            .unwrap_err()
            .cause()
            .unwrap()
            .to_string()
            .contains("missing field")
    );
}

#[test]
fn succeed_to_encode_response() {
    let response = HttpResponse::Ok().cbor(response::Logout);
    assert!(response.is_ok());
}
