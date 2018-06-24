//! The Main Content component

use frontend::services::{
    cookie::CookieService,
    protocol::ProtocolService,
    websocket::{WebSocketService, WebSocketStatus},
};
use yew::{prelude::*, services::ConsoleService};
use SESSION_COOKIE;

/// Data Model for the Content component
pub struct ContentComponent {
    cookie_service: CookieService,
    console_service: ConsoleService,
    protocol_service: ProtocolService,
    websocket_service: WebSocketService,
    button_disabled: bool,
}

#[derive(Debug)]
/// Available message types to process
pub enum Message {
    LogoutRequest,
    LogoutResponse(Vec<u8>),
    WebSocketConnected,
    WebSocketFailure,
}

impl Component for ContentComponent {
    type Message = Message;
    type Properties = ();

    /// Initialization routine
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.send_back(|data| Message::LogoutResponse(data));
        let notification = link.send_back(|data| match data {
            WebSocketStatus::Opened => Message::WebSocketConnected,
            _ => Message::WebSocketFailure,
        });

        // Create the component
        Self {
            cookie_service: CookieService::new(),
            console_service: ConsoleService::new(),
            protocol_service: ProtocolService::new(),
            websocket_service: WebSocketService::new_with_callbacks(callback, notification)
                .expect("No valid websocket connection"),
            button_disabled: true,
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
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
                    } else {
                        self.console_service.error("Unable to write logout request");
                    }
                } else {
                    self.console_service.error("No session cookie found");
                }
            }
            Message::LogoutResponse(mut response) => match self.protocol_service.read_logout_response(&mut response) {
                Ok(()) => self.cookie_service.remove(SESSION_COOKIE),
                Err(e) => self.console_service.error(&format!("Unable to logout: {}", e)),
            },
            Message::WebSocketConnected => {
                self.console_service.info("Websocket connected");
                self.button_disabled = false;
            }
            Message::WebSocketFailure => {
                self.console_service.warn("Lost websocket connection");
                self.button_disabled = true;
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
                <button disabled=self.button_disabled,
                        class="uk-button uk-button-default",
                        onclick=|_| Message::LogoutRequest,>{"Logout"}</button>
            </div>
        }
    }
}
