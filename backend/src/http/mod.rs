//! HTTP message handling

pub mod login_credentials;
pub mod login_session;
pub mod logout;
mod tests;

pub use http::{login_credentials::login_credentials, login_session::login_session, logout::logout};

use actix_web::{error::Error, HttpResponse};
use futures::Future;

pub type FutureResponse = Box<Future<Item = HttpResponse, Error = Error>>;
