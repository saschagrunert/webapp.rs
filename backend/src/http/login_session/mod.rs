//! The session based login request

use actix::{dev::ToEnvelope, prelude::*};
use actix_web::{AsyncResponder, HttpRequest, HttpResponse};
use cbor::CborResponseBuilder;
use database::UpdateSession;
use futures::Future;
use http::{unpack_cbor, FutureResponse};
use server::State;
use token::Token;
use webapp::protocol::{model::Session, request::LoginSession, response::Login};

mod test;

pub fn login_session<T>(http_request: &HttpRequest<State<T>>) -> FutureResponse
where
    T: Actor + Handler<UpdateSession>,
    <T as Actor>::Context: ToEnvelope<T, UpdateSession>,
{
    let (request_clone, cbor) = unpack_cbor(http_request);
    // Create a new token for the already given one
    cbor.and_then(|LoginSession(Session {token})| {
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
                .and_then(|result| Ok(HttpResponse::Ok().cbor(Login(result?))?))
        })
        .responder()
}
