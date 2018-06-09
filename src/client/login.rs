//! The Login component
use yew::prelude::*;

/// Data Model for the Login component
pub struct LoginComponent {}

impl<C> Component<C> for LoginComponent {
    type Message = ();
    type Properties = ();

    fn create(_: (), _: &mut Env<C, Self>) -> Self {
        LoginComponent {}
    }

    fn update(&mut self, _: Self::Message, _: &mut Env<C, Self>) -> ShouldRender {
        true
    }
}

impl<C: 'static> Renderable<C, LoginComponent> for LoginComponent {
    fn view(&self) -> Html<C, Self> {
        html! {
            <h2>{"Login"}</h2>
        }
    }
}

