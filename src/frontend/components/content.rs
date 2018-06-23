//! The Main Content component

use frontend::services::{cookie::CookieService, protocol::ProtocolService, websocket::WebSocketService};
use yew::prelude::*;
use SESSION_COOKIE;

/// Data Model for the Content component
pub struct ContentComponent {
    cookie_service: CookieService,
    protocol_service: ProtocolService,
    websocket_service: WebSocketService,
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
            protocol_service: ProtocolService::new(),
            websocket_service: WebSocketService::new().expect("No valid websocket connection"),
        }
    }

    /// Called everytime when messages are received
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::LogoutRequest => {
                // Retrieve the currently set cookie
                if let Ok(token) = self.cookie_service.get(SESSION_COOKIE) {
                    // Create the logout request
                    if let Ok(data) = self.protocol_service.write_logout_request(&token) {
                        // Send the request
                        self.websocket_service.send(data);
                    }
                    // Remove the cookie if set
                    self.cookie_service.remove(SESSION_COOKIE);
                }
            }
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
