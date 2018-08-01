//! The credential based login request

use actix::{dev::ToEnvelope, prelude::*};
use actix_web::{error::ErrorUnauthorized, AsyncResponder, HttpRequest, HttpResponse};
use cbor::CborResponseBuilder;
use database::CreateSession;
use futures::Future;
use http::{unpack_cbor, FutureResponse};
use server::State;
use token::Token;
use webapp::protocol::{request, response};

mod test;

pub fn login_credentials<T>(http_request: &HttpRequest<State<T>>) -> FutureResponse
where
    T: Actor + Handler<CreateSession>,
    <T as Actor>::Context: ToEnvelope<T, CreateSession>,
{
    let (request_clone, cbor) = unpack_cbor(http_request);
    // Verify username and password
    cbor.and_then(|request::LoginCredentials{username, password}| {
            debug!("User {} is trying to login", username);
            if username.is_empty() || password.is_empty() || username != password {
                return Err(ErrorUnauthorized("wrong username or password"));
            }
            Ok(username)
        })
        // Create a new token
        .and_then(|username| Ok(Token::create(&username)?))
        // Update the session in the database
        .and_then(move |token| {
            request_clone
                .state()
                .database
                .send(CreateSession(token))
                .from_err()
                .and_then(|result| Ok(HttpResponse::Ok().cbor(response::Login(result?))?))
        })
        .responder()
}
