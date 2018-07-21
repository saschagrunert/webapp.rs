//! All available routes within this application for fragment based routing

use services::router::Route;
use std::convert::Into;

macro_rules! routes {
    ($(($x:tt, $y:expr)),*) => {
        #[derive(Debug, PartialEq)]
        /// Possible child components of this one
        pub enum RouterComponent {
            $($x,)*
        }

        /// Convert a RouterComponent into a Route
        impl<T> Into<Route<T>> for RouterComponent where T: Default {
            fn into(self) -> Route<T> {
                Route {
                    fragment: Some(
                        match self {
                            $(RouterComponent::$x => $y,)*
                        }.into(),
                    ),
                    ..Default::default()
                }
            }
        }

        /// Convert a Route into a RouterComponent
        impl<T> Into<RouterComponent> for Route<T> {
            fn into(self) -> RouterComponent {
                match self.fragment {
                    Some(f) => match f.as_str() {
                        $($y => RouterComponent::$x,)*
                        _ => RouterComponent::Error,
                    },
                    _ => RouterComponent::Error,
                }
            }
        }
    };
}

/// Available routes
routes!(
    (Error, "/error"),
    (Loading, "/loading"),
    (Login, "/login"),
    (Register, "/register"),
    (Content, "/content")
);
