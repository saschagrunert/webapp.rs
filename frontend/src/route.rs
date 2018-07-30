//! All available routes within this application for fragment based routing

use service::router::Route;
use std::convert::Into;

macro_rules! routes {
    ($($x:tt => $y:expr,)*) => (
        #[derive(Debug, PartialEq)]
        /// Possible child components of this one
        pub enum RouterTarget {
            $($x,)*
        }

        /// Convert a RouterTarget into a Route
        impl<T> Into<Route<T>> for RouterTarget where T: Default {
            fn into(self) -> Route<T> {
                Route {
                    fragment: Some(
                        match self {
                            $(RouterTarget::$x => $y,)*
                        }.into(),
                    ),
                    ..Default::default()
                }
            }
        }

        /// Convert a Route into a RouterTarget
        impl<T> Into<RouterTarget> for Route<T> {
            fn into(self) -> RouterTarget {
                match self.fragment {
                    Some(f) => match f.as_str() {
                        $($y => RouterTarget::$x,)*
                        _ => RouterTarget::Error,
                    },
                    _ => RouterTarget::Error,
                }
            }
        }
    )
}

/// Available routes
routes! {
    Error => "/error",
    Loading => "/loading",
    Login =>  "/login",
    Content => "/content",
}
