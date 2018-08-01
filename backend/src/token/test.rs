//! Everything related to web token handling

#![cfg(test)]

use token::Token;

#[test]
fn succeed_to_create_a_token() {
    assert!(Token::create("").is_ok());
}

#[test]
fn succeed_to_verify_a_token() {
    let sut = Token::create("").unwrap();
    assert!(Token::verify(&sut).is_ok());
}

#[test]
fn fail_to_verify_a_wrong_token() {
    assert!(Token::verify("wrong").is_err());
}
