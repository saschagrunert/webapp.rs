//! The Main Content component

use frontend::{
    routes::RouterComponent,
    services::{
        cookie::CookieService,
        protocol::ProtocolService,
        router::{Request, RouterAgent},
        websocket::WebSocketService,
    },
};
use yew::{prelude::*, services::ConsoleService};
use SESSION_COOKIE;

/// Data Model for the Content component
pub struct ContentComponent {
    router_agent: Box<Bridge<RouterAgent<()>>>,
    cookie_service: CookieService,
    console_service: ConsoleService,
    protocol_service: ProtocolService,
    websocket_service: WebSocketService,
    button_disabled: bool,
}

/// Available message types to process
pub enum Message {
    Ignore,
    LogoutRequest,
    LogoutResponse(Vec<u8>),
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
            router_agent.send(Request::ChangeRoute(RouterComponent::Login.into()));
        }

        // Create the component
        Self {
            router_agent,
            cookie_service,
            console_service,
            protocol_service: ProtocolService::new(),
            websocket_service: WebSocketService::new(
                link.send_back(|data| Message::LogoutResponse(data)),
                link.send_back(|_| Message::Ignore),
            ),
            button_disabled: false,
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }

    /// Called everytime when messages are received
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::Ignore => {}
            Message::LogoutRequest => {
                // Retrieve the currently set cookie
                if let Ok(token) = self.cookie_service.get(SESSION_COOKIE) {
                    // Create the logout request
                    if let Ok(data) = self.protocol_service.write_request_logout(&token) {
                        // Disable user interaction
                        self.button_disabled = true;

                        // Send the request
                        self.websocket_service.send(data);
                    } else {
                        self.console_service.error("Unable to write logout request");
                    }
                } else {
                    self.console_service.error("No session cookie found");
                }
            }
            Message::LogoutResponse(mut response) => match self.protocol_service.read_response_logout(&mut response) {
                Ok(Some(())) => {
                    self.console_service.log("Got valid logout response");
                    self.cookie_service.remove(SESSION_COOKIE);
                    self.router_agent
                        .send(Request::ChangeRoute(RouterComponent::Login.into()));
                }
                Ok(None) => {} // Not my response
                Err(e) => self.console_service.info(&format!("Unable to logout: {}", e)),
            },
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
