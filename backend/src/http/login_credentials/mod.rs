//! The credential based login request

use actix::{dev::ToEnvelope, prelude::*};
use actix_web::{
    error::{ErrorInternalServerError, ErrorUnauthorized},
    AsyncResponder, HttpRequest, HttpResponse,
};
use cbor::{CborRequest, CborResponseBuilder};
use database::CreateSession;
use futures::Future;
use http::FutureResponse;
use server::State;
use token::Token;
use webapp::protocol::{request, response};

mod tests;

pub fn login_credentials<T>(http_request: &HttpRequest<State<T>>) -> FutureResponse
where
    T: Actor + Handler<CreateSession>,
    <T as Actor>::Context: ToEnvelope<T, CreateSession>,
{
    let request_clone = http_request.clone();
    CborRequest::new(http_request)
        .from_err()
        // Verify username and password
        .and_then(|request::LoginCredentials{username, password}| {
            debug!("User {} is trying to login", username);
            if username.is_empty() || password.is_empty() || username != password {
                return Err(ErrorUnauthorized("wrong username or password"));
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
