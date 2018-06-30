//! The Login component

use frontend::{
    routes::RouterComponent,
    services::{
        cookie::CookieService,
        protocol::ProtocolService,
        router::{Request, Route, RouterAgent},
        websocket::WebSocketService,
    },
};
use yew::{prelude::*, services::ConsoleService};
use SESSION_COOKIE;

/// Data Model for the Login component
pub struct LoginComponent {
    router_agent: Box<Bridge<RouterAgent<()>>>,
    username: String,
    password: String,
    button_disabled: bool,
    cookie_service: CookieService,
    console_service: ConsoleService,
    protocol_service: ProtocolService,
    websocket_service: WebSocketService,
}

/// Available message types to process
pub enum Message {
    HandleRoute(Route<()>),
    LoginRequest,
    LoginResponse(Vec<u8>),
    UpdateUsername(String),
    UpdatePassword(String),
    WebSocketIgnore,
}

impl Component for LoginComponent {
    type Message = Message;
    type Properties = ();

    /// Initialization routine
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            router_agent: RouterAgent::bridge(link.send_back(|route| Message::HandleRoute(route))),
            username: String::new(),
            password: String::new(),
            button_disabled: true,
            cookie_service: CookieService::new(),
            console_service: ConsoleService::new(),
            websocket_service: WebSocketService::new(
                link.send_back(|data| Message::LoginResponse(data)),
                link.send_back(|_| Message::WebSocketIgnore),
            ),
            protocol_service: ProtocolService::new(),
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }

    /// Called everytime when messages are received
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::LoginRequest => match self
                .protocol_service
                .write_request_login_credential(&self.username, &self.password)
            {
                Ok(data) => {
                    // Disable user interaction
                    self.button_disabled = true;

                    // Send the request
                    self.websocket_service.send(data);
                    false
                }
                Err(e) => {
                    self.console_service
                        .error(&format!("Unable to create login credential request: {}", e));
                    false
                }
            },
            Message::LoginResponse(mut response) => match self.protocol_service.read_response_login(&mut response) {
                Ok(Some(token)) => {
                    self.console_service.info("Login succeed");

                    // Set the retrieved session cookie
                    self.cookie_service.set(SESSION_COOKIE, &token);

                    // Route to the next component
                    self.router_agent
                        .send(Request::ChangeRoute(RouterComponent::Content.into()));

                    true
                }
                Ok(None) => false, // Not my response
                Err(e) => {
                    self.console_service.error(&format!("Unable to login: {}", e));
                    self.button_disabled = false;
                    true
                }
            },
            Message::UpdateUsername(new_username) => {
                self.username = new_username;
                self.update_button_state();
                true
            }
            Message::UpdatePassword(new_password) => {
                self.password = new_password;
                self.update_button_state();
                true
            }
            _ => true,
        }
    }
}

impl LoginComponent {
    fn update_button_state(&mut self) {
        self.button_disabled = self.username.is_empty() || self.password.is_empty();
    }
}

impl Renderable<LoginComponent> for LoginComponent {
    fn view(&self) -> Html<Self> {
        html! {
            <div class="uk-card uk-card-default uk-card-body uk-width-1-3@s uk-position-center",>
                <form onsubmit="return false",>
                    <fieldset class="uk-fieldset",>
                        <legend class="uk-legend",>{"Authentication"}</legend>
                        <div class="uk-margin",>
                            <input class="uk-input",
                                   placeholder="Username",
                                   value=&self.username,
                                   oninput=|e| Message::UpdateUsername(e.value), />
                        </div>
                        <div class="uk-margin",>
                            <input class="uk-input",
                                   type="password",
                                   placeholder="Password",
                                   value=&self.password,
                                   oninput=|e| Message::UpdatePassword(e.value), />
                        </div>
                        <button class="uk-button uk-button-default",
                                type="submit",
                                disabled=self.button_disabled,
                                onclick=|_| Message::LoginRequest,>{"Login"}</button>
                    </fieldset>
                </form>
            </div>
        }
    }
}
