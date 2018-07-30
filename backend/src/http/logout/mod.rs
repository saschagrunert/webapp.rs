//! The logout request

use actix::{dev::ToEnvelope, prelude::*};
use actix_web::{AsyncResponder, HttpRequest, HttpResponse};
use cbor::{CborRequest, CborResponseBuilder};
use database::DeleteSession;
use futures::Future;
use http::FutureResponse;
use server::State;
use webapp::protocol::{model::Session, request, response};

mod tests;

pub fn logout<T: Actor>(http_request: &HttpRequest<State<T>>) -> FutureResponse
where
    T: Actor + Handler<DeleteSession>,
    <T as Actor>::Context: ToEnvelope<T, DeleteSession>,
{
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
                .and_then(|result| {
                    result?;
                    Ok(HttpResponse::Ok().cbor(response::Logout)?)
                })
        })
        .responder()
}
