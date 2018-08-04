//! Api related helpers and utilities

use failure::Error;
use yew::{format::Cbor, services::fetch::Response as FetchResponse};

/// A generic response type of the API
pub type Response<T> = FetchResponse<Cbor<Result<T, Error>>>;

#[macro_export]
/// Generic API access macro
macro_rules! api {
    ($url:expr) => {
        env!("API_URL").to_owned() + $url
    };
}

#[macro_export]
/// Generic API fetch macro
macro_rules! fetch {
    ($request:expr => $api:expr, $link:expr, $msg:expr, $succ:expr, $err:expr) => {
        match ::yew::services::fetch::Request::post(api!($api)).body(Cbor(&$request)) {
            Ok(body) => {
                $succ();
                Some(
                    ::yew::services::fetch::FetchService::new()
                        .fetch_binary(body, $link.send_back($msg)),
                )
            }
            Err(_) => {
                $err();
                None
            }
        };
    };
}
