//! The registration component

use yew::prelude::*;

/// Data Model for the Register component
pub struct RegisterComponent {}

/// Available message types to process
pub enum Message {}

impl Component for RegisterComponent {
    type Message = Message;
    type Properties = ();

    /// Initialization routine
    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        // Create the component
        Self {}
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }

    /// Called everytime when messages are received
    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }
}

impl Renderable<RegisterComponent> for RegisterComponent {
    fn view(&self) -> Html<Self> {
        html! {
            <div class="uk-card uk-card-default uk-card-body uk-width-1-3@s uk-position-center",>
                {"Register Component"}
            </div>
        }
    }
}
