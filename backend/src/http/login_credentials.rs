//! The credential based login request

use crate::{
    database::{CreateSession, DatabaseExecutor},
    token::Token,
};
use actix::prelude::*;
use actix_web::{
    error::{ErrorInternalServerError, ErrorUnauthorized},
    web::{Data, Json},
    Error, HttpResponse,
};
use log::debug;
use webapp::protocol::{request::LoginCredentials, response::Login};

pub async fn login_credentials(
    payload: Json<LoginCredentials>,
    database: Data<Addr<DatabaseExecutor>>,
) -> Result<HttpResponse, Error> {
    let r = payload.into_inner();

    debug!("User {} is trying to login", r.username);
    // Verify username and password
    if r.username.is_empty() || r.password.is_empty() || r.username != r.password {
        return Err(ErrorUnauthorized("wrong username or password"));
    }

    // Create a new token
    match Token::create(&r.username) {
        Ok(token) => {
            // Update the session in the database
            let result = database
                .send(CreateSession(token))
                .await
                .map_err(ErrorInternalServerError)?;
            Ok(HttpResponse::Ok().json(Login(result.map_err(ErrorInternalServerError)?)))
        }
        Err(e) => Err(e.into()),
    }
}
