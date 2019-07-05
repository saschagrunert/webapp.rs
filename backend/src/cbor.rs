//! Cbor abstraction for HTTP message handling

use actix_web::{
    dev::HttpResponseBuilder, error::Error as HttpError, http::header::CONTENT_TYPE, web::Payload,
    HttpResponse, ResponseError,
};
use bytes::BytesMut;
use failure::{format_err, Error, Fail};
use futures::{Future, Poll, Stream};
use serde::{de::DeserializeOwned, Serialize};
use serde_cbor::{error::Error as SerdeError, from_slice, to_vec};

#[derive(Debug, Fail)]
pub enum CborError {
    #[fail(display = "Payload read error: {}", _0)]
    /// Payload error
    Payload(#[cause] Error),

    #[fail(display = "Serialization error: {}", _0)]
    /// Serialize error
    Serialize(#[cause] SerdeError),

    #[fail(display = "Deserialization error: {}", _0)]
    /// Deserialize error
    Deserialize(#[cause] SerdeError),
}

impl ResponseError for CborError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::BadRequest().into()
    }
}

impl From<SerdeError> for CborError {
    fn from(err: SerdeError) -> CborError {
        CborError::Deserialize(err)
    }
}

/// A wrapped request based on a future
pub struct CborRequest<T>(Box<Future<Item = T, Error = CborError>>);

impl<T> CborRequest<T>
where
    T: DeserializeOwned + 'static,
{
    pub fn new(body: Payload) -> Self {
        CborRequest(Box::new(
            body.map_err(|e| CborError::Payload(format_err!("{}", e)))
                .fold(BytesMut::new(), move |mut body, chunk| {
                    body.extend_from_slice(&chunk);
                    Ok::<_, CborError>(body)
                })
                .and_then(|body| Ok(from_slice(&body)?)),
        ))
    }
}

impl<T> Future for CborRequest<T> {
    type Error = CborError;
    type Item = T;

    fn poll(&mut self) -> Poll<T, CborError> {
        self.0.poll()
    }
}

pub trait CborResponseBuilder {
    fn cbor<T: Serialize>(&mut self, value: T) -> Result<HttpResponse, HttpError>;
}

impl CborResponseBuilder for HttpResponseBuilder {
    fn cbor<T: Serialize>(&mut self, value: T) -> Result<HttpResponse, HttpError> {
        self.header(CONTENT_TYPE, "application/cbor");
        let body = to_vec(&value).map_err(CborError::Serialize)?;
        Ok(self.body(body))
    }
}
