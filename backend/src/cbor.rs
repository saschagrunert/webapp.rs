//! Cbor abstraction for HTTP message handling

use actix_web::{
    dev::HttpResponseBuilder,
    error::{Error as HttpError, PayloadError},
    http::header::CONTENT_TYPE,
    HttpMessage, HttpRequest, HttpResponse, ResponseError,
};
use bytes::BytesMut;
use futures::{Future, Poll, Stream};
use serde_cbor::{error::Error as SerdeError, from_slice, to_vec};
use webapp::protocol::{Request, Response};

const DEFAULT_CONTENT_TYPE: &str = "application/cbor";

#[derive(Fail, Debug)]
pub enum CborError {
    #[fail(display = "Payload read error: {}", _0)]
    /// Payload error
    Payload(#[cause] PayloadError),

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

/// A wrapped Request based on a future
pub struct CborRequest(Box<Future<Item = Request, Error = CborError>>);

impl CborRequest {
    pub fn new<S>(req: &HttpRequest<S>) -> Self {
        let future = req
            .payload()
            .map_err(CborError::Payload)
            .fold(BytesMut::new(), move |mut body, chunk| {
                body.extend_from_slice(&chunk);
                Ok::<_, CborError>(body)
            })
            .and_then(|body| Ok(from_slice(&body)?));

        CborRequest(Box::new(future))
    }
}

impl Future for CborRequest {
    type Item = Request;
    type Error = CborError;

    fn poll(&mut self) -> Poll<Request, CborError> {
        self.0.poll()
    }
}

pub trait CborResponseBuilder {
    fn cbor(&mut self, value: Response) -> Result<HttpResponse, HttpError>;
}

impl CborResponseBuilder for HttpResponseBuilder {
    fn cbor(&mut self, value: Response) -> Result<HttpResponse, HttpError> {
        self.header(CONTENT_TYPE, DEFAULT_CONTENT_TYPE);
        let body = to_vec(&value).map_err(CborError::Serialize)?;
        Ok(self.body(body))
    }
}
