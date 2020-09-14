//! The logout request

use crate::database::{DatabaseExecutor, DeleteSession};
use actix::prelude::*;
use actix_web::{
    error::ErrorInternalServerError,
    web::{Data, Json},
    Error, HttpResponse,
};
use log::debug;
use webapp::protocol::{request, response};

pub async fn logout(
    payload: Json<request::Logout>,
    database: Data<Addr<DatabaseExecutor>>,
) -> Result<HttpResponse, Error> {
    let token = payload.into_inner().0.token;

    // Remove the session from the database
    debug!("Session token {} wants to be logged out", token);
    let _ = database
        .send(DeleteSession(token))
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(response::Logout))
}
