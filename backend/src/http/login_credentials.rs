//! The credential based login request

use crate::{
    cbor::{CborRequest, CborResponseBuilder},
    database::{CreateSession, DatabaseExecutor},
    token::Token,
};
use actix::prelude::*;
use actix_web::{
    error::ErrorUnauthorized,
    web::{Data, Payload},
    Error, HttpResponse,
};
use futures::Future;
use log::debug;
use webapp::protocol::{request::LoginCredentials, response::Login};

pub fn login_credentials(
    payload: Payload,
    database: Data<Addr<DatabaseExecutor>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let cbor = CborRequest::new(payload);

    // Verify username and password
    cbor.from_err().and_then(|LoginCredentials{username, password}| {
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
                database
                .send(CreateSession(token))
                .from_err()
                .and_then(|result| Ok(HttpResponse::Ok().cbor(Login(result?))?))
        })
}
