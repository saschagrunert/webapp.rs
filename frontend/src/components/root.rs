//! The Root component

use components::{content::ContentComponent, login::LoginComponent, register::RegisterComponent};
use routes::RouterComponent;
use serde_cbor::from_slice;
use services::{
    cookie::CookieService,
    router::{self, Route, RouterAgent},
    uikit::{NotificationStatus, UIkitService},
    websocket::{WebSocketAgent, WebSocketRequest, WebSocketResponse},
};
use webapp::protocol::{Login, Request, Response, Session};
use yew::{prelude::*, services::ConsoleService};
use SESSION_COOKIE;

/// Data Model for the Root Component
pub struct RootComponent {
    router_agent: Box<Bridge<RouterAgent<()>>>,
    websocket_agent: Box<Bridge<WebSocketAgent>>,
    child_component: RouterComponent,
    cookie_service: CookieService,
    console_service: ConsoleService,
    uikit_service: UIkitService,
}

/// Available message types to process
pub enum Message {
    HandleRoute(Route<()>),
    Ws(WebSocketResponse),
}

impl Component for RootComponent {
    type Message = Message;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            router_agent: RouterAgent::bridge(link.send_back(Message::HandleRoute)),
            websocket_agent: WebSocketAgent::bridge(link.send_back(Message::Ws)),
            child_component: RouterComponent::Loading,
            console_service: ConsoleService::new(),
            cookie_service: CookieService::new(),
            uikit_service: UIkitService::new(),
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::HandleRoute(route) => {
                self.child_component = route.into();
            }
            Message::Ws(WebSocketResponse::Opened) => {
                // Verify if a session cookie already exist and try to authenticate if so
                if let Ok(token) = self.cookie_service.get(SESSION_COOKIE) {
                    match Request::Login(Login::Session(Session { token })).to_vec() {
                        Some(data) => {
                            self.console_service.info("Token found, trying to authenticate");
                            self.websocket_agent.send(WebSocketRequest(data));
                        }
                        None => {
                            self.cookie_service.remove(SESSION_COOKIE);
                            self.router_agent
                                .send(router::Request::ChangeRoute(RouterComponent::Login.into()));
                        }
                    }
                } else {
                    self.console_service.info("No token found, routing to login");
                    self.router_agent
                        .send(router::Request::ChangeRoute(RouterComponent::Login.into()));
                }
            }
            Message::Ws(WebSocketResponse::Data(response)) => match from_slice(&response) {
                Ok(Response::LoginSession(Ok(Session { token }))) => {
                    self.console_service.info("Session based login succeed");

                    // Set the retrieved session cookie
                    self.cookie_service.set(SESSION_COOKIE, &token);

                    // Route to the content component
                    self.router_agent
                        .send(router::Request::ChangeRoute(RouterComponent::Content.into()));
                }
                Ok(Response::LoginSession(Err(e))) => {
                    // Remote the existing cookie
                    self.console_service.info(&format!("Session based login failed: {}", e));
                    self.cookie_service.remove(SESSION_COOKIE);
                    self.router_agent
                        .send(router::Request::ChangeRoute(RouterComponent::Login.into()));
                }
                Ok(Response::Error) => {
                    // Send a notification to the user and route to the error page
                    self.uikit_service
                        .notify("Internal server error", &NotificationStatus::Danger);
                    self.router_agent
                        .send(router::Request::ChangeRoute(RouterComponent::Error.into()));
                }
                _ => {} // Not my response
            },
            Message::Ws(WebSocketResponse::Error) => {
                // Send a notification to the user
                self.uikit_service
                    .notify("Server connection unavailable", &NotificationStatus::Danger);

                // Route to the error child if coming from the loading child
                if self.child_component == RouterComponent::Loading {
                    self.router_agent
                        .send(router::Request::ChangeRoute(RouterComponent::Error.into()));
                }
            }
            Message::Ws(WebSocketResponse::Closed) => {
                // Send a notification to the user if app already in usage
                if self.child_component != RouterComponent::Error {
                    self.uikit_service
                        .notify("Server connection closed", &NotificationStatus::Danger);
                }
            }
        }
        true
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
                <div class="uk-position-center", uk-icon="icon: cloud-download; ratio: 3",></div>
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
                <div class="uk-position-center", uk-icon="icon: ban; ratio: 3",></div>
            },
        }
    }
}
