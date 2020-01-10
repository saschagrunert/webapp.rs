//! Api related helpers and utilities

use failure::Fallible;
use yew::{format::Json, services::fetch::Response as FetchResponse};

/// A generic response type of the API
pub type Response<T> = FetchResponse<Json<Fallible<T>>>;

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
                Some(
                    ::yew::services::fetch::FetchService::new()
                        .fetch_binary(body, $link.callback($msg)),
                )
            }
            Err(_) => {
                $err();
                None
            }
        };
    };
}
