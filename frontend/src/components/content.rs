//! The Main Content component

use routes::RouterComponent;
use serde_cbor::from_slice;
use services::{
    cookie::CookieService,
    router::{self, RouterAgent},
    websocket::{WebSocketAgent, WebSocketRequest, WebSocketResponse},
};
use webapp::protocol::{self, Response, Session};
use yew::{prelude::*, services::ConsoleService};
use SESSION_COOKIE;

/// Data Model for the Content component
pub struct ContentComponent {
    router_agent: Box<Bridge<RouterAgent<()>>>,
    websocket_agent: Box<Bridge<WebSocketAgent>>,
    cookie_service: CookieService,
    console_service: ConsoleService,
    logout_button_disabled: bool,
}

/// Available message types to process
pub enum Message {
    Ignore,
    LogoutRequest,
    Ws(WebSocketResponse),
}

impl Component for ContentComponent {
    type Message = Message;
    type Properties = ();

    /// Initialization routine
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        // Guard the authentication
        let mut router_agent = RouterAgent::bridge(link.send_back(|_| Message::Ignore));
        let cookie_service = CookieService::new();
        let mut console_service = ConsoleService::new();
        if cookie_service.get(SESSION_COOKIE).is_err() {
            console_service.log("No session token found, routing back to login");
            router_agent.send(router::Request::ChangeRoute(RouterComponent::Login.into()));
        }

        // Create the component
        Self {
            router_agent,
            websocket_agent: WebSocketAgent::bridge(link.send_back(|r| Message::Ws(r))),
            cookie_service,
            console_service,
            logout_button_disabled: false,
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
                    match protocol::Request::Logout(Session { token: token }).to_vec() {
                        Some(data) => {
                            // Disable user interaction
                            self.logout_button_disabled = true;

                            // Send the request
                            self.websocket_agent.send(WebSocketRequest(data));
                        }
                        None => self.console_service.error("Unable to write logout request"),
                    }
                } else {
                    // It should not happen but in case there is no session cookie on logout, route
                    // back to login
                    self.console_service.error("No session cookie found");
                    self.router_agent
                        .send(router::Request::ChangeRoute(RouterComponent::Login.into()));
                }
            }
            Message::Ws(WebSocketResponse::Data(response)) => match from_slice(&response) {
                Ok(Response::Logout(Ok(()))) => {
                    self.console_service.log("Got valid logout response");
                    self.cookie_service.remove(SESSION_COOKIE);
                    self.router_agent
                        .send(router::Request::ChangeRoute(RouterComponent::Login.into()));
                }
                Ok(Response::Logout(Err(e))) => self.console_service.info(&format!("Unable to logout: {}", e)),
                _ => {} // Not my response
            },
            Message::Ignore | Message::Ws(WebSocketResponse::Opened) => {}
            Message::Ws(WebSocketResponse::Error) | Message::Ws(WebSocketResponse::Closed) => {
                self.logout_button_disabled = true
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
                <button disabled=self.logout_button_disabled,
                    class="uk-button uk-button-default",
                    onclick=|_| Message::LogoutRequest,>{"Logout"}</button>
            </div>
        }
    }
}
