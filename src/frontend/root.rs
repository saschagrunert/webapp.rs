//! The Root component
use frontend::{
    login::LoginComponent,
    services::{
        cookie::CookieService,
        protocol::ProtocolService,
        websocket::{WebSocketService, WebSocketStatus},
    },
};
use yew::prelude::*;
use SESSION_COOKIE;

#[derive(Debug)]
/// Available message types to process
pub enum Message {
    LoginRequest(String),
    LoginResponse(Vec<u8>),
    WebSocketConnected,
    WebSocketIgnore,
}

/// Data Model for the Root Component
pub struct RootComponent {
    authentication_state: AuthenticationState,
    cookie_service: CookieService,
    protocol_service: ProtocolService,
    websocket_service: WebSocketService,
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
        let callback = link.send_back(|data| Message::LoginResponse(data));
        let notification = link.send_back(|data| match data {
            WebSocketStatus::Opened => Message::WebSocketConnected,
            _ => Message::WebSocketIgnore,
        });
        let websocket_service = WebSocketService::new(callback, notification).expect("No valid websocket connection");

        Self {
            authentication_state: AuthenticationState::Unknown,
            cookie_service: CookieService::new(),
            protocol_service: ProtocolService::new(),
            websocket_service,
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::WebSocketConnected => {
                // Verify if a session cookie already exist and try to authenticate if so
                if let Ok(token) = self.cookie_service.get_cookie(SESSION_COOKIE) {
                    match self.protocol_service.write_login_token_request(&token) {
                        Ok(data) => {
                            self.websocket_service.send(data);
                            false
                        }
                        Err(_) => {
                            self.cookie_service.remove_cookie(SESSION_COOKIE);
                            self.authentication_state = AuthenticationState::UnAuthenticated;
                            true
                        }
                    }
                } else {
                    self.authentication_state = AuthenticationState::UnAuthenticated;
                    true
                }
            }
            Message::LoginResponse(mut response) => match self.protocol_service.read_login_response(&mut response) {
                Ok(token) => {
                    // Set the retrieved session cookie
                    self.cookie_service.set_cookie(SESSION_COOKIE, &token);
                    self.authentication_state = AuthenticationState::Authenticated;
                    true
                }
                Err(_) => {
                    self.cookie_service.remove_cookie(SESSION_COOKIE);
                    self.authentication_state = AuthenticationState::UnAuthenticated;
                    true
                }
            },
            _ => false,
        }
    }
}

impl Renderable<RootComponent> for RootComponent {
    fn view(&self) -> Html<Self> {
        match self.authentication_state {
            AuthenticationState::Unknown => html! {
                <div></div>
            },
            AuthenticationState::Authenticated => html! {
               <h1>{"Already authenticated"}</h1>
            },
            AuthenticationState::UnAuthenticated => html! {
               <LoginComponent:/>
            },
        }
    }
}
