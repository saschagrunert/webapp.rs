//! All available routes within this application

use frontend::services::router::Route;
use std::convert::Into;

/// Possible child components of this one
pub enum RouterComponent {
    Content,
    Error,
    Loading,
    Login,
}

/// Convert a RouterComponent into a Route
impl<T> Into<Route<T>> for RouterComponent
where
    T: Default,
{
    fn into(self) -> Route<T> {
        Route {
            fragment: Some(
                match self {
                    RouterComponent::Content => "content",
                    RouterComponent::Error => "error",
                    RouterComponent::Loading => "loading",
                    RouterComponent::Login => "login",
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
                "content" => RouterComponent::Content,
                "loading" => RouterComponent::Loading,
                "login" => RouterComponent::Login,
                _ => RouterComponent::Error,
            },
            _ => RouterComponent::Error,
        }
    }
}
