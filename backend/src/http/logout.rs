//! The logout request

use crate::{
    cbor::{CborRequest, CborResponseBuilder},
    database::{DatabaseExecutor, DeleteSession},
};
use actix::prelude::*;
use actix_web::{
    web::{Data, Payload},
    Error, HttpResponse,
};
use futures::Future;
use log::debug;
use webapp::protocol::{model::Session, request, response};

pub fn logout(
    payload: Payload,
    database: Data<Addr<DatabaseExecutor>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let cbor = CborRequest::new(payload);

    // Remove the session from the database
    cbor.from_err()
        .and_then(move |request::Logout(Session { token })| {
            debug!("Session token {} wants to be logged out", token);
            database
                .send(DeleteSession(token))
                .from_err()
                .and_then(|result| {
                    result?;
                    Ok(HttpResponse::Ok().cbor(response::Logout)?)
                })
        })
}
