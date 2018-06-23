//! The Main Content component

use frontend::services::cookie::CookieService;
use yew::prelude::*;
use SESSION_COOKIE;

/// Data Model for the Content component
pub struct ContentComponent {
    cookie_service: CookieService,
}

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
        Self {
            cookie_service: CookieService::new(),
        }
    }

    /// Called everytime when messages are received
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::LogoutRequest => self.cookie_service.remove_cookie(SESSION_COOKIE),
        }
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
