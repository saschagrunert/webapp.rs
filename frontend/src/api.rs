//! Api related helpers and utilities

use anyhow::Result;
use yew::{format::Json, services::fetch::Response as FetchResponse};

/// A generic response type of the API
pub type Response<T> = FetchResponse<Json<Result<T>>>;

#[macro_export]
/// Generic API fetch macro
macro_rules! fetch {
    ($request:expr => $api:expr, $link:expr, $msg:expr, $succ:expr, $err:expr) => {
        match ::yew::services::fetch::Request::post(env!("API_URL").to_owned() + $api)
            .header("Content-Type", "application/json")
            .body(Json(&$request))
        {
            Ok(body) => {
                $succ();
                ::yew::services::fetch::FetchService::fetch_binary(body, $link.callback($msg)).ok()
            }
            Err(_) => {
                $err();
                None
            }
        };
    };
}
