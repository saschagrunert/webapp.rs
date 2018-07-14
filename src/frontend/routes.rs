//! All available routes within this application

use frontend::services::router::Route;
use std::convert::Into;

macro_rules! routes {
    ($($x:tt),*) => {
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
                            $(RouterComponent::$x => stringify!($x),)*
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
                        $(stringify!($x) => RouterComponent::$x,)*
                        _ => RouterComponent::Error,
                    },
                    _ => RouterComponent::Error,
                }
            }
        }
    };
}

/// Available routes
routes!(Error, Loading, Login, Register, Content);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn succeed_to_convert_loading() {
        let sut: Route<()> = Route {
            fragment: Some("Loading".to_owned()),
            ..Default::default()
        };
        let cmp: RouterComponent = sut.clone().into();
        assert_eq!(cmp, RouterComponent::Loading);
        assert_eq!(sut, RouterComponent::Loading.into());
    }

    #[test]
    fn succeed_to_convert_login() {
        let sut: Route<()> = Route {
            fragment: Some("Login".to_owned()),
            ..Default::default()
        };
        let cmp: RouterComponent = sut.clone().into();
        assert_eq!(cmp, RouterComponent::Login);
        assert_eq!(sut, RouterComponent::Login.into());
    }

    #[test]
    fn succeed_to_convert_register() {
        let sut: Route<()> = Route {
            fragment: Some("Register".to_owned()),
            ..Default::default()
        };
        let cmp: RouterComponent = sut.clone().into();
        assert_eq!(cmp, RouterComponent::Register);
        assert_eq!(sut, RouterComponent::Register.into());
    }

    #[test]
    fn succeed_to_convert_content() {
        let sut: Route<()> = Route {
            fragment: Some("Content".to_owned()),
            ..Default::default()
        };
        let cmp: RouterComponent = sut.clone().into();
        assert_eq!(cmp, RouterComponent::Content);
        assert_eq!(sut, RouterComponent::Content.into());
    }

    #[test]
    fn succeed_to_convert_error() {
        let sut: Route<()> = Route {
            fragment: Some("error".to_owned()),
            ..Default::default()
        };
        let cmp: RouterComponent = sut.clone().into();
        assert_eq!(cmp, RouterComponent::Error);
        assert_eq!(sut, RouterComponent::Error.into());
    }

    #[test]
    fn succeed_to_convert_unknown() {
        let sut: Route<()> = Route {
            fragment: Some("new_route".to_owned()),
            ..Default::default()
        };
        let cmp: RouterComponent = sut.into();
        assert_eq!(cmp, RouterComponent::Error);
    }
}
