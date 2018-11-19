//! Everything related to web token handling

#![cfg(test)]

use crate::token::Token;
use failure::Fallible;

#[test]
fn succeed_to_create_a_token() {
    // Given
    // When
    // Then
    assert!(Token::create("").is_ok());
}

#[test]
fn succeed_to_verify_a_token() -> Fallible<()> {
    // Given
    let sut = Token::create("")?;

    // When
    // Then
    assert!(Token::verify(&sut).is_ok());
    Ok(())
}

#[test]
fn fail_to_verify_a_wrong_token() {
    // Given
    // When
    // Then
    assert!(Token::verify("wrong").is_err());
}
