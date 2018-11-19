//! The credential based login request

use actix::{dev::ToEnvelope, prelude::*};
use actix_web::{error::ErrorUnauthorized, AsyncResponder, HttpRequest, HttpResponse};
use crate::cbor::CborResponseBuilder;
use crate::database::CreateSession;
use futures::Future;
use crate::http::{unpack_cbor, FutureResponse};
use crate::server::State;
use crate::token::Token;
use webapp::protocol::{request::LoginCredentials, response::Login};

mod test;

pub fn login_credentials<T>(http_request: &HttpRequest<State<T>>) -> FutureResponse
where
    T: Actor + Handler<CreateSession>,
    <T as Actor>::Context: ToEnvelope<T, CreateSession>,
{
    let (request_clone, cbor) = unpack_cbor(http_request);
    // Verify username and password
    cbor.and_then(|LoginCredentials{username, password}| {
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
                .and_then(|result| Ok(HttpResponse::Ok().cbor(Login(result?))?))
        })
        .responder()
}
