//! The Main Content component
use yew::prelude::*;

/// Data Model for the Content component
pub struct ContentComponent {}

#[derive(Debug)]
/// Available message types to process
pub enum Message {
    LogoutRequest,
}

impl Component for ContentComponent {
    type Message = Message;
    type Properties = ();

    /// Initialization routine
    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        // Create the component
        Self {}
    }

    /// Called everytime when messages are received
    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }
}

impl Renderable<ContentComponent> for ContentComponent {
    fn view(&self) -> Html<Self> {
        html! {
            <div class="uk-card uk-card-default uk-card-body uk-width-1-3@s uk-position-center",>
                <h1 class="uk-card-title",>{"Content"}</h1>
                <button class="uk-button uk-button-default", onclick=|_| Message::LogoutRequest,>{"Logout"}</button>
            </div>
        }
    }
}
