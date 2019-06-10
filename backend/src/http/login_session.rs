//! The session based login request

use crate::{
    cbor::{CborRequest, CborResponseBuilder},
    database::{DatabaseExecutor, UpdateSession},
    token::Token,
};
use actix::prelude::*;
use actix_web::{
    web::{Data, Payload},
    Error, HttpResponse,
};
use futures::Future;
use log::debug;
use webapp::protocol::{model::Session, request::LoginSession, response::Login};

pub fn login_session(
    payload: Payload,
    database: Data<Addr<DatabaseExecutor>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let cbor = CborRequest::new(payload);

    // Create a new token for the already given one
    cbor.from_err().and_then(|LoginSession(Session {token})| {
            debug!("Session token {} wants to be renewed", token);
            Ok((Token::verify(&token)?, token))
        })
        // Update the session in the database
        .and_then(move |(new_token, old_token)| {
                database
                .send(UpdateSession { old_token, new_token })
                .from_err()
                .and_then(|result| Ok(HttpResponse::Ok().cbor(Login(result?))?))
        })
}
