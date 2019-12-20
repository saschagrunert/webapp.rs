//! Cbor extractor and responder

use failure::Fail;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::{fmt, ops};

use bytes::BytesMut;
use futures::future::{err, ok, FutureExt, LocalBoxFuture, Ready};
use futures::StreamExt;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_cbor::{error::Error as SerdeError, from_slice, to_vec};

use actix_http::http::{header::CONTENT_LENGTH, StatusCode};
use actix_http::{HttpMessage, error::PayloadError, Payload, Response};
use actix_web::{FromRequest, Error, Responder, HttpRequest};
use mime::Mime;

pub struct Cbor<T>(pub T);


#[derive(Debug, Fail)]
pub enum CborError {
     #[fail(display = "Payload read error: {}", _0)]
    /// Payload error
    Payload(#[cause] PayloadError),

    #[fail(display = "Maximum payload size reached")]
    /// Maximum payload size reached
    Overflow,

    #[fail(display = "Serialization error: {}", _0)]
    /// Serialize error
    Serialize(#[cause] SerdeError),

    #[fail(display = "Deserialization error: {}", _0)]
    /// Deserialize error
    Deserialize(#[cause] SerdeError),
}

impl<T> Cbor<T> {
    /// Deconstruct to an inner value
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> ops::Deref for Cbor<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> ops::DerefMut for Cbor<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> fmt::Debug for Cbor<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cbor: {:?}", self.0)
    }
}

impl<T> fmt::Display for Cbor<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}


impl<T: Serialize> Responder for Cbor<T> {
    type Error = Error;
    type Future = Ready<Result<Response, Error>>;

    fn respond_to(self, _: &HttpRequest) -> Self::Future {
        let body = match to_vec(&self.0) {
            Ok(body) => body,
            Err(e) => return err(e.into()),
        };

        ok(Response::build(StatusCode::OK)
            .content_type("application/cbor")
            .body(body))
    }
}

impl<T> FromRequest for Cbor<T>
where
    T: DeserializeOwned + 'static,
{
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self, Error>>;
    type Config = JsonConfig;

    #[inline]
    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let req2 = req.clone();
        let (limit, err, ctype) = req
            .app_data::<Self::Config>()
            .map(|c| (c.limit, c.ehandler.clone(), c.content_type.clone()))
            .unwrap_or((32768, None, None));

        CborBody::new(req, payload, ctype)
            .limit(limit)
            .map(move |res| match res {
                Err(e) => {
                    log::debug!(
                        "Failed to deserialize Cbor from payload. \
                         Request path: {}",
                        req2.path()
                    );
                    if let Some(err) = err {
                        Err((*err)(e, &req2))
                    } else {
                        Err(e.into())
                    }
                }
                Ok(data) => Ok(Cbor(data)),
            })
            .boxed_local()
    }
}

#[derive(Clone)]
pub struct JsonConfig {
    limit: usize,
    ehandler: Option<Arc<dyn Fn(CborError, &HttpRequest) -> Error + Send + Sync>>,
    content_type: Option<Arc<dyn Fn(Mime) -> bool + Send + Sync>>,
}

impl JsonConfig {
    /// Change max size of payload. By default max size is 32Kb
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Set custom error handler
    pub fn error_handler<F>(mut self, f: F) -> Self
    where
        F: Fn(CborError, &HttpRequest) -> Error + Send + Sync + 'static,
    {
        self.ehandler = Some(Arc::new(f));
        self
    }

    /// Set predicate for allowed content types
    pub fn content_type<F>(mut self, predicate: F) -> Self
    where
        F: Fn(Mime) -> bool + Send + Sync + 'static,
    {
        self.content_type = Some(Arc::new(predicate));
        self
    }
}

impl Default for JsonConfig {
    fn default() -> Self {
        JsonConfig {
            limit: 32768,
            ehandler: None,
            content_type: None,
        }
    }
}

pub struct CborBody<U> {
    limit: usize,
    length: Option<usize>,
    stream: Option<Payload>,
    err: Option<CborError>,
    fut: Option<LocalBoxFuture<'static, Result<U, CborError>>>,
}

impl<U> CborBody<U>
where
    U: DeserializeOwned + 'static,
{
    /// Create `CborBody` for request.
    pub fn new(
        req: &HttpRequest,
        payload: &mut Payload,
        ctype: Option<Arc<dyn Fn(Mime) -> bool + Send + Sync>>,
    ) -> Self {
        let len = req
            .headers()
            .get(&CONTENT_LENGTH)
            .and_then(|l| l.to_str().ok())
            .and_then(|s| s.parse::<usize>().ok());

        let payload = payload.take();
        CborBody {
            limit: 262_144,
            length: len,
            stream: Some(payload),
            fut: None,
            err: None,
        }
    }

    /// Change max size of payload. By default max size is 256Kb
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
}

impl<U> Future for CborBody<U>
where
    U: DeserializeOwned + 'static,
{
    type Output = Result<U, CborError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(ref mut fut) = self.fut {
            return Pin::new(fut).poll(cx);
        }

        if let Some(err) = self.err.take() {
            return Poll::Ready(Err(err));
        }

        let limit = self.limit;
        if let Some(len) = self.length.take() {
            if len > limit {
                return Poll::Ready(Err(CborError::Overflow));
            }
        }
        let mut stream = self.stream.take().unwrap();

        self.fut = Some(
            async move {
                let mut body = BytesMut::with_capacity(8192);

                while let Some(item) = stream.next().await {
                    let chunk = item.map_err(|e| CborError::Payload(e))?;
                    if (body.len() + chunk.len()) > limit {
                        return Err(CborError::Overflow);
                    } else {
                        body.extend_from_slice(&chunk);
                    }
                }
                Ok(from_slice::<U>(&body).map_err(|e| CborError::Deserialize(e))?)
            }
                .boxed_local(),
        );

        self.poll(cx)
    }
}
