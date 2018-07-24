//! HTTP message handling

use actix_web::{
    error::{Error as HttpError, ErrorBadRequest, ErrorForbidden, ErrorInternalServerError},
    AsyncResponder, HttpRequest, HttpResponse,
};
use cbor::{CborRequest, CborResponseBuilder};
use database::{CreateSession, DeleteSession, UpdateSession};
use futures::Future;
use server::State;
use token::Token;
use webapp::protocol::{request, response, Request, Response};

type FutureResponse = Box<Future<Item = HttpResponse, Error = HttpError>>;

/// Returns a generic bad request on receiving a wrong message type
fn wrong_message_type<T>() -> Result<T, HttpError> {
    Err(ErrorBadRequest("wrong message type"))
}

pub fn login_credentials(http_request: &HttpRequest<State>) -> FutureResponse {
    let request_clone = http_request.clone();
    CborRequest::new(http_request)
        .from_err()
        // Extract username and password
        .and_then(|request: Request| match request {
            Request::Login(request::Login::Credentials{username, password}) => {
                debug!("User {} is trying to login", username);
                Ok((username, password))
            },
            // When it is not the correct request
            _ => wrong_message_type(),
        })
        // Verify username and password
        .and_then(|(username, password)| {
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
                    Ok(r) => Ok(HttpResponse::Ok().cbor(Response::Login(response::Login::Credentials(Ok(r))))?),
                    Err(_) => Ok(HttpResponse::InternalServerError().into()),
                })
        })
        .responder()
}

pub fn login_session(http_request: &HttpRequest<State>) -> FutureResponse {
    let request_clone = http_request.clone();
    CborRequest::new(http_request)
        .from_err()
        // Extract the session token
        .and_then(|request: Request| match request {
            Request::Login(request::Login::Session(session)) => {
                debug!("Session token {} wants to be renewed", session.token);
                Ok(session.token)
            },
            // When it is not the correct request
            _ => wrong_message_type(),
        })
        // Create a new token for the already given one
        .and_then(|token| {
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
                    Ok(r) => Ok(HttpResponse::Ok().cbor(Response::Login(response::Login::Session(Ok(r))))?),
                    Err(_) => Ok(HttpResponse::InternalServerError().into()),
                })
        })
        .responder()
}

pub fn logout(http_request: &HttpRequest<State>) -> FutureResponse {
    let request_clone = http_request.clone();
    CborRequest::new(http_request)
        .from_err()
        // Extract the session token
        .and_then(|request: Request| match request {
            Request::Logout(session) => {
                debug!("Session token {} wants to be logged out", session.token);
                Ok(session.token)
            },
            // When it is not the correct request
            _ => wrong_message_type(),
        })
        // Remove the session from the database
        .and_then(move |token| {
            request_clone
                .state()
                .database
                .send(DeleteSession(token))
                .from_err()
                .and_then(|result| match result {
                    Ok(()) => Ok(HttpResponse::Ok().cbor(Response::Logout(Ok(())))?),
                    Err(_) => Ok(HttpResponse::InternalServerError().into()),
                })
        })
        .responder()
}
