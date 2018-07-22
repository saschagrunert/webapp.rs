//! HTTP message handling

use actix_web::error::{Error as ActixError, ErrorBadRequest};
use actix_web::{AsyncResponder, HttpRequest, HttpResponse};
use cbor::{CborRequest, CborResponseBuilder};
// use database::DeleteSession;
use futures::Future;
use server::State;
use webapp::protocol::{Request, Response};

pub fn logout(http_request: &HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = ActixError>> {
    CborRequest::new(http_request)
        .from_err()
        .and_then(|request: Request| match request {
            Request::Logout(_session) => {
                // Remove the session from the internal storage
                /* http_request
                    .state()
                    .database
                    .send(DeleteSession("asd".to_owned()))
                    .wait()??; */

                Ok(HttpResponse::Ok().cbor(Response::Logout(Ok(())))?)
            }
            // When it is not the correct request
            _ => Err(ErrorBadRequest("wrong message type")),
        })
        .responder()
}
