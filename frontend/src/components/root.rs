//! The Root component

use components::{content::ContentComponent, login::LoginComponent, register::RegisterComponent};
use routes::RouterComponent;
use serde_cbor::from_slice;
use services::{
    cookie::CookieService,
    router::{self, Route, RouterAgent},
    uikit::{NotificationStatus, UIkitService},
    websocket::{WebSocketService, WebSocketStatus},
};
use webapp::protocol::{self, Login, Session};
use yew::{prelude::*, services::ConsoleService};
use SESSION_COOKIE;

/// Available message types to process
pub enum Message {
    HandleRoute(Route<()>),
    WebSocketClosed,
    WebSocketOpened,
    WebSocketError,
    WebSocketResponse(Vec<u8>),
}

/// Data Model for the Root Component
pub struct RootComponent {
    router_agent: Box<Bridge<RouterAgent<()>>>,
    child_component: RouterComponent,
    cookie_service: CookieService,
    console_service: ConsoleService,
    uikit_service: UIkitService,
    websocket_service: WebSocketService,
}

impl Component for RootComponent {
    type Message = Message;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            router_agent: RouterAgent::bridge(link.send_back(|route| Message::HandleRoute(route))),
            child_component: RouterComponent::Loading,
            console_service: ConsoleService::new(),
            cookie_service: CookieService::new(),
            uikit_service: UIkitService::new(),
            websocket_service: WebSocketService::new(
                link.send_back(|data| Message::WebSocketResponse(data)),
                link.send_back(|data| match data {
                    WebSocketStatus::Closed => Message::WebSocketClosed,
                    WebSocketStatus::Opened => Message::WebSocketOpened,
                    WebSocketStatus::Error => Message::WebSocketError,
                }),
            ),
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::HandleRoute(route) => {
                self.child_component = route.into();
                true
            }
            Message::WebSocketOpened => {
                // Verify if a session cookie already exist and try to authenticate if so
                if let Ok(token) = self.cookie_service.get(SESSION_COOKIE) {
                    match protocol::Request::Login(Login::Session(Session { token: token })).to_vec() {
                        Some(data) => {
                            self.console_service.info("Token found, trying to authenticate");
                            self.websocket_service.send(&data);
                            false
                        }
                        None => {
                            self.cookie_service.remove(SESSION_COOKIE);
                            self.router_agent
                                .send(router::Request::ChangeRoute(RouterComponent::Login.into()));
                            true
                        }
                    }
                } else {
                    self.console_service.info("No token found, routing to login");
                    self.router_agent
                        .send(router::Request::ChangeRoute(RouterComponent::Login.into()));
                    true
                }
            }
            Message::WebSocketResponse(response) => match from_slice(&response) {
                Ok(protocol::Response::Login(Ok(Session { token }))) => {
                    // Set the retrieved session cookie
                    self.console_service.info("Login succeed");
                    self.cookie_service.set(SESSION_COOKIE, &token);
                    self.router_agent
                        .send(router::Request::ChangeRoute(RouterComponent::Content.into()));
                    true
                }
                Ok(protocol::Response::Login(Err(e))) => {
                    // Remote the existing cookie
                    self.console_service.info(&format!("Login failed: {}", e));
                    self.cookie_service.remove(SESSION_COOKIE);
                    self.router_agent
                        .send(router::Request::ChangeRoute(RouterComponent::Login.into()));
                    true
                }
                Ok(protocol::Response::Error) => {
                    self.router_agent
                        .send(router::Request::ChangeRoute(RouterComponent::Error.into()));
                    true
                }
                _ => false, // Not my response
            },
            Message::WebSocketError => {
                // Send a notification to the user
                self.uikit_service
                    .notify("Server connection unavailable", NotificationStatus::Danger);

                // Route to the error child if coming from the loading child
                if self.child_component == RouterComponent::Loading {
                    self.router_agent
                        .send(router::Request::ChangeRoute(RouterComponent::Error.into()));
                }
                true
            }
            Message::WebSocketClosed => {
                // Send a notification to the user if app already in usage
                if self.child_component != RouterComponent::Error {
                    self.uikit_service
                        .notify("Server connection closed", NotificationStatus::Danger);
                }
                true
            }
        }
    }
}

impl Renderable<RootComponent> for RootComponent {
    fn view(&self) -> Html<Self> {
        self.child_component.view()
    }
}

impl Renderable<RootComponent> for RouterComponent {
    fn view(&self) -> Html<RootComponent> {
        match *self {
            RouterComponent::Loading => html! {
                <div class="uk-position-center", uk-spinner="",></div>
            },
            RouterComponent::Login => html! {
               <LoginComponent:/>
            },
            RouterComponent::Register => html! {
               <RegisterComponent:/>
            },
            RouterComponent::Content => html! {
               <ContentComponent:/>
            },
            RouterComponent::Error => html! {
                <div class="uk-position-center",>
                    {"Error loading application."}
                </div>
            },
        }
    }
}
