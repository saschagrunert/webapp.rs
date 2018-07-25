//! HTTP message handling

use actix_web::{
    error::{Error as HttpError, ErrorForbidden, ErrorInternalServerError},
    AsyncResponder, HttpRequest, HttpResponse,
};
use cbor::{CborRequest, CborResponseBuilder};
use database::{CreateSession, DeleteSession, UpdateSession};
use futures::Future;
use server::State;
use token::Token;
use webapp::protocol::{model::Session, request, response};

type FutureResponse = Box<Future<Item = HttpResponse, Error = HttpError>>;

pub fn login_credentials(http_request: &HttpRequest<State>) -> FutureResponse {
    let request_clone = http_request.clone();
    CborRequest::new(http_request)
        .from_err()
        // Verify username and password
        .and_then(|request::LoginCredentials{username, password}| {
            debug!("User {} is trying to login", username);
            if username.is_empty() || password.is_empty() || username != password {
                return Err(ErrorForbidden("wrong username or password"));
            }
            Ok(username)
        })
        // Create a new token
        .and_then(|username| {
            Token::create(&username).map_err(|_| {
                 ErrorInternalServerError("token creation failed")
            })
        })
        // Update the session in the database
        .and_then(move |token| {
            request_clone
                .state()
                .database
                .send(CreateSession(token))
                .from_err()
                .and_then(|result| match result {
                    Ok(r) => Ok(HttpResponse::Ok().cbor(response::Login(r))?),
                    Err(_) => Ok(HttpResponse::InternalServerError().into()),
                })
        })
        .responder()
}

pub fn login_session(http_request: &HttpRequest<State>) -> FutureResponse {
    let request_clone = http_request.clone();
    CborRequest::new(http_request)
        .from_err()
        // Create a new token for the already given one
        .and_then(|request::LoginSession(Session{token})| {
            debug!("Session token {} wants to be renewed", token);
            Token::verify(&token).map_err(|_| {
                 ErrorForbidden("Token verification failed")
            }).and_then(|new_token| {
                 Ok((token, new_token))
            })
        })
        // Update the session in the database
        .and_then(move |(old_token, new_token)| {
            request_clone
                .state()
                .database
                .send(UpdateSession { old_token, new_token })
                .from_err()
                .and_then(|result| match result {
                    Ok(r) => Ok(HttpResponse::Ok().cbor(response::Login(r))?),
                    Err(_) => Ok(HttpResponse::InternalServerError().into()),
                })
        })
        .responder()
}

pub fn logout(http_request: &HttpRequest<State>) -> FutureResponse {
    let request_clone = http_request.clone();
    CborRequest::new(http_request)
        .from_err()
        // Remove the session from the database
        .and_then(move |request::Logout(Session{token})| {
            debug!("Session token {} wants to be logged out", token);
            request_clone
                .state()
                .database
                .send(DeleteSession(token))
                .from_err()
                .and_then(|result| match result {
                    Ok(()) => Ok(HttpResponse::Ok().cbor(response::Logout)?),
                    Err(_) => Ok(HttpResponse::InternalServerError().into()),
                })
        })
        .responder()
}
