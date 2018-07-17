//! The Login component

use routes::RouterComponent;
use serde_cbor::from_slice;
use services::{
    cookie::CookieService,
    router::{self, RouterAgent},
    uikit::{NotificationStatus, UIkitService},
    websocket::WebSocketService,
};
use webapp::protocol::{self, Login, Response, Session};
use yew::{prelude::*, services::ConsoleService};
use SESSION_COOKIE;

/// Data Model for the Login component
pub struct LoginComponent {
    router_agent: Box<Bridge<RouterAgent<()>>>,
    username: String,
    password: String,
    login_button_disabled: bool,
    inputs_and_register_button_disabled: bool,
    cookie_service: CookieService,
    console_service: ConsoleService,
    uikit_service: UIkitService,
    websocket_service: WebSocketService,
}

/// Available message types to process
pub enum Message {
    Ignore,
    LoginRequest,
    RegisterRequest,
    UpdatePassword(String),
    UpdateUsername(String),
    WebSocketResponse(Vec<u8>),
    WebSocketError,
}

impl Component for LoginComponent {
    type Message = Message;
    type Properties = ();

    /// Initialization routine
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            router_agent: RouterAgent::bridge(link.send_back(|_| Message::Ignore)),
            username: String::new(),
            password: String::new(),
            login_button_disabled: true,
            inputs_and_register_button_disabled: false,
            cookie_service: CookieService::new(),
            console_service: ConsoleService::new(),
            uikit_service: UIkitService::new(),
            websocket_service: WebSocketService::new(
                link.send_back(|data| Message::WebSocketResponse(data)),
                link.send_back(|_| Message::WebSocketError),
            ),
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }

    /// Called everytime when messages are received
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::Ignore => {}
            Message::WebSocketError => {
                self.login_button_disabled = true;
                self.inputs_and_register_button_disabled = true;
            }
            Message::LoginRequest => {
                match protocol::Request::Login(Login::Credentials {
                    username: self.username.to_owned(),
                    password: self.password.to_owned(),
                }).to_vec()
                {
                    Some(data) => {
                        // Disable user interaction
                        self.login_button_disabled = true;
                        self.inputs_and_register_button_disabled = true;

                        // Send the request
                        self.websocket_service.send(&data);
                    }
                    None => {
                        self.console_service.error("Unable to create login credential request");
                    }
                }
            }
            Message::WebSocketResponse(response) => match from_slice(&response) {
                Ok(Response::Login(Ok(Session { token }))) => {
                    self.console_service.info("Login succeed");

                    // Set the retrieved session cookie
                    self.cookie_service.set(SESSION_COOKIE, &token);

                    // Route to the content component
                    self.router_agent
                        .send(router::Request::ChangeRoute(RouterComponent::Content.into()));
                }
                Ok(Response::Login(Err(e))) => {
                    self.console_service.warn(&format!("Unable to login: {}", e));
                    self.uikit_service
                        .notify("Authentication failed", NotificationStatus::Warning);
                    self.login_button_disabled = false;
                    self.inputs_and_register_button_disabled = false;
                }
                _ => {} // Not my response
            },
            Message::UpdateUsername(new_username) => {
                self.username = new_username;
                self.update_button_state();
            }
            Message::UpdatePassword(new_password) => {
                self.password = new_password;
                self.update_button_state();
            }
            Message::RegisterRequest => {
                // Route to the register component
                self.router_agent
                    .send(router::Request::ChangeRoute(RouterComponent::Register.into()));
            }
        }
        true
    }
}

impl LoginComponent {
    fn update_button_state(&mut self) {
        self.login_button_disabled = self.username.is_empty() || self.password.is_empty();
    }
}

impl Renderable<LoginComponent> for LoginComponent {
    fn view(&self) -> Html<Self> {
        html! {
            <div class="uk-card uk-card-default uk-card-body uk-width-1-3@s uk-position-center",>
                <form onsubmit="return false",>
                    <fieldset class="uk-fieldset",>
                        <legend class="uk-legend",>{"Login"}</legend>
                        <input class="uk-input uk-margin",
                            placeholder="Username",
                            disabled=self.inputs_and_register_button_disabled,
                            value=&self.username,
                            oninput=|e| Message::UpdateUsername(e.value), />
                        <input class="uk-input uk-margin-bottom",
                            type="password",
                            placeholder="Password",
                            disabled=self.inputs_and_register_button_disabled,
                            value=&self.password,
                            oninput=|e| Message::UpdatePassword(e.value), />
                        <div class="uk-button-group",>
                            <button class="uk-button uk-button-primary",
                                type="submit",
                                disabled=self.login_button_disabled,
                                onclick=|_| Message::LoginRequest,>{"Login"}</button>
                            <button class="uk-button uk-button-default",
                                type="register",
                                disabled=self.inputs_and_register_button_disabled,
                                onclick=|_| Message::RegisterRequest,>{"Register"}</button>
                        </div>
                    </fieldset>
                </form>
            </div>
        }
    }
}
