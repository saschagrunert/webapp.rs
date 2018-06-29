//! The Root component

use frontend::{
    components::{content::ContentComponent, login::LoginComponent},
    routes::RouterComponent,
    services::{
        cookie::CookieService,
        protocol::ProtocolService,
        router::{Request, Route, RouterAgent},
        websocket::{WebSocketService, WebSocketStatus},
    },
};
use yew::{prelude::*, services::ConsoleService};
use SESSION_COOKIE;

/// Available message types to process
pub enum Message {
    HandleRoute(Route<()>),
    LoginRequest(String),
    LoginResponse(Vec<u8>),
    WebSocketConnected,
    WebSocketFailure,
}

/// Data Model for the Root Component
pub struct RootComponent {
    router_agent: Box<Bridge<RouterAgent<()>>>,
    child_component: RouterComponent,
    cookie_service: CookieService,
    console_service: ConsoleService,
    protocol_service: ProtocolService,
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
            protocol_service: ProtocolService::new(),
            websocket_service: WebSocketService::new(
                link.send_back(|data| Message::LoginResponse(data)),
                link.send_back(|data| match data {
                    WebSocketStatus::Opened => Message::WebSocketConnected,
                    _ => Message::WebSocketFailure,
                }),
            ),
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::WebSocketConnected => {
                // Verify if a session cookie already exist and try to authenticate if so
                if let Ok(token) = self.cookie_service.get(SESSION_COOKIE) {
                    match self.protocol_service.write_login_token_request(&token) {
                        Ok(data) => {
                            self.console_service.info("Token found, trying to authenticate");
                            self.websocket_service.send(data);
                            false
                        }
                        Err(_) => {
                            self.cookie_service.remove(SESSION_COOKIE);
                            self.router_agent
                                .send(Request::ChangeRoute(RouterComponent::Login.into()));
                            true
                        }
                    }
                } else {
                    self.console_service.info("No token found, routing to login");
                    self.router_agent
                        .send(Request::ChangeRoute(RouterComponent::Login.into()));
                    true
                }
            }
            Message::LoginResponse(mut response) => match self.protocol_service.read_login_response(&mut response) {
                Ok(Some(token)) => {
                    // Set the retrieved session cookie
                    self.console_service.info("Login succeed");
                    self.cookie_service.set(SESSION_COOKIE, &token);
                    self.router_agent
                        .send(Request::ChangeRoute(RouterComponent::Content.into()));
                    true
                }
                Ok(None) => false, // Not my response
                Err(e) => {
                    // Remote the existing cookie
                    self.console_service.info(&format!("Login failed: {}", e));
                    self.cookie_service.remove(SESSION_COOKIE);
                    self.router_agent
                        .send(Request::ChangeRoute(RouterComponent::Login.into()));
                    true
                }
            },
            Message::HandleRoute(route) => {
                self.child_component = route.into();
                true
            }
            _ => {
                self.router_agent
                    .send(Request::ChangeRoute(RouterComponent::Error.into()));
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
            RouterComponent::Error => html! {
                <div class="uk-position-center",>
                    {"Error loading application."}
                </div>
            },
            RouterComponent::Login => html! {
               <LoginComponent:/>
            },
            RouterComponent::Content => html! {
               <ContentComponent:/>
            },
        }
    }
}
