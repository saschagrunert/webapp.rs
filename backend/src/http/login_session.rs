//! The session based login request

use crate::{
    database::{DatabaseExecutor, UpdateSession},
    token::Token,
};
use actix::prelude::*;
use actix_web::{
    error::ErrorInternalServerError,
    web::{Data, Json},
    Error, HttpResponse,
};
use log::debug;
use webapp::protocol::{request::LoginSession, response::Login};

pub async fn login_session(
    payload: Json<LoginSession>,
    database: Data<Addr<DatabaseExecutor>>,
) -> Result<HttpResponse, Error> {
    let old_token = payload.into_inner().0.token;

    // Create a new token for the already given one
    debug!("Session token {} wants to be renewed", old_token);
    let new_token = Token::verify(&old_token)?;

    // Update the session in the database
    let result = database
        .send(UpdateSession {
            old_token,
            new_token,
        })
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(Login(result.map_err(ErrorInternalServerError)?)))
}
