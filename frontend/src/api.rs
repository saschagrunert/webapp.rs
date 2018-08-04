//! Api related helpers and utilities

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
        match fetch::Request::post(api!($api)).body(Cbor(&$request)) {
            Ok(body) => {
                $succ();
                Some(FetchService::new().fetch_binary(body, $link.send_back($msg)))
            }
            Err(_) => {
                $err();
                None
            }
        };
    };
}
