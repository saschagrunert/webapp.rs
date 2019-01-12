//! The logout request

use crate::{
    cbor::CborResponseBuilder,
    database::DeleteSession,
    http::{unpack_cbor, FutureResponse},
    server::State,
};
use actix::{dev::ToEnvelope, prelude::*};
use actix_web::{AsyncResponder, HttpRequest, HttpResponse};
use futures::Future;
use log::debug;
use webapp::protocol::{model::Session, request, response};

mod test;

pub fn logout<T: Actor>(http_request: &HttpRequest<State<T>>) -> FutureResponse
where
    T: Actor + Handler<DeleteSession>,
    <T as Actor>::Context: ToEnvelope<T, DeleteSession>,
{
    let (request_clone, cbor) = unpack_cbor(http_request);
    // Remove the session from the database
    cbor.and_then(move |request::Logout(Session { token })| {
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
