//! The Root component

use frontend::{
    components::content::ContentComponent,
    components::login::LoginComponent,
    services::{
        cookie::CookieService,
        protocol::ProtocolService,
        router::{Route, RouterAgent},
        websocket::{WebSocketService, WebSocketStatus},
    },
};
use yew::{prelude::*, services::ConsoleService};
use SESSION_COOKIE;

#[derive(Debug)]
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
    authentication_state: AuthenticationState,
    initial_message: String,
    cookie_service: CookieService,
    console_service: ConsoleService,
    protocol_service: ProtocolService,
    websocket_service: WebSocketService,
    router_agent: Box<Bridge<RouterAgent<()>>>,
    child_component: ChildComponent,
}

/// Possible child components of this one
enum ChildComponent {
    Content,
    Error,
    Loading,
    Login,
}

/// Possible authentication states
enum AuthenticationState {
    Unknown,
    Authenticated,
    UnAuthenticated,
}

impl Component for RootComponent {
    type Message = Message;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            authentication_state: AuthenticationState::Unknown,
            initial_message: "Loading application…".to_owned(),
            console_service: ConsoleService::new(),
            cookie_service: CookieService::new(),
            protocol_service: ProtocolService::new(),
            websocket_service: WebSocketService::new_with_callbacks(
                link.send_back(|data| Message::LoginResponse(data)),
                link.send_back(|data| match data {
                    WebSocketStatus::Opened => Message::WebSocketConnected,
                    _ => Message::WebSocketFailure,
                }),
            ).expect("No valid websocket connection"),
            router_agent: RouterAgent::bridge(link.send_back(|route| Message::HandleRoute(route))),
            child_component: ChildComponent::Loading,
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
                            self.authentication_state = AuthenticationState::UnAuthenticated;
                            true
                        }
                    }
                } else {
                    self.console_service.info("No token found");
                    self.authentication_state = AuthenticationState::UnAuthenticated;
                    true
                }
            }
            Message::LoginResponse(mut response) => match self.protocol_service.read_login_response(&mut response) {
                Ok(token) => {
                    // Set the retrieved session cookie
                    self.console_service.info("Login succeed");
                    self.cookie_service.set(SESSION_COOKIE, &token);
                    self.authentication_state = AuthenticationState::Authenticated;
                    true
                }
                Err(_) => {
                    // Remote the existing cookie
                    self.console_service.info("Login failed");
                    self.cookie_service.remove(SESSION_COOKIE);
                    self.authentication_state = AuthenticationState::UnAuthenticated;
                    true
                }
            },
            Message::HandleRoute(route) => {
                if let Some(first_segment) = route.path_segments.get(0) {
                    self.child_component = match first_segment.as_str() {
                        "content" => ChildComponent::Content,
                        "login" => ChildComponent::Login,
                        "" => ChildComponent::Loading,
                        _ => ChildComponent::Error,
                    }
                }
                true
            }
            _ => {
                self.initial_message = "Error loading application.".to_owned();
                true
            }
        }
    }
}

impl Renderable<RootComponent> for RootComponent {
    fn view(&self) -> Html<Self> {
        match self.authentication_state {
            AuthenticationState::Unknown => html! {
                <div class="uk-position-center",>
                    {&self.initial_message}
                </div>
            },
            AuthenticationState::Authenticated => html! {
               <ContentComponent:/>
            },
            AuthenticationState::UnAuthenticated => html! {
               <LoginComponent:/>
            },
        }
    }
}

impl Renderable<RootComponent> for ChildComponent {
    fn view(&self) -> Html<RootComponent> {
        match *self {
            ChildComponent::Loading => html! {
                <div class="uk-position-center",>
                    {"Loading application…"}
                </div>
            },
            ChildComponent::Error => html! {
                <div class="uk-position-center",>
                    {"Error loading application."}
                </div>
            },
            ChildComponent::Login => html! {
               <LoginComponent:/>
            },
            ChildComponent::Content => html! {
               <ContentComponent:/>
            },
        }
    }
}
