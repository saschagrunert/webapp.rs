//! The Root component
use yew::prelude::*;
use client::login::LoginComponent;

/// Data Model for the Root Component
pub struct RootComponent {}

impl<C> Component<C> for RootComponent {
    type Message = ();
    type Properties = ();

    fn create(_: (), _: &mut Env<C, Self>) -> Self {
        RootComponent {}
    }

    fn update(&mut self, _: Self::Message, _: &mut Env<C, Self>) -> ShouldRender {
        true
    }
}

impl<C: 'static> Renderable<C, RootComponent> for RootComponent {
    fn view(&self) -> Html<C, Self> {
        html! {
            <LoginComponent:/>
        }
    }
}
