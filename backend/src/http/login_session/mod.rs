//! The session based login request

use actix::{dev::ToEnvelope, prelude::*};
use actix_web::{AsyncResponder, HttpRequest, HttpResponse};
use cbor::{CborRequest, CborResponseBuilder};
use database::UpdateSession;
use futures::Future;
use http::FutureResponse;
use server::State;
use token::Token;
use webapp::protocol::{model::Session, request, response};

mod tests;

pub fn login_session<T>(http_request: &HttpRequest<State<T>>) -> FutureResponse
where
    T: Actor + Handler<UpdateSession>,
    <T as Actor>::Context: ToEnvelope<T, UpdateSession>,
{
    let request_clone = http_request.clone();
    CborRequest::new(http_request)
        .from_err()
        // Create a new token for the already given one
        .and_then(|request::LoginSession(Session{token})| {
            debug!("Session token {} wants to be renewed", token);
            Ok((Token::verify(&token)?, token))
        })
        // Update the session in the database
        .and_then(move |(new_token, old_token)| {
            request_clone
                .state()
                .database
                .send(UpdateSession { old_token, new_token })
                .from_err()
                .and_then(|result| Ok(HttpResponse::Ok().cbor(response::Login(result?))?))
        })
        .responder()
}
