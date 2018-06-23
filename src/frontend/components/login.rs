//! The Login component

use frontend::services::{cookie::CookieService, protocol::ProtocolService, websocket::WebSocketService};
use yew::{prelude::*, services::ConsoleService};
use SESSION_COOKIE;

/// Data Model for the Login component
pub struct LoginComponent {
    username: String,
    password: String,
    button_disabled: bool,
    cookie_service: CookieService,
    console_service: ConsoleService,
    protocol_service: ProtocolService,
    websocket_service: WebSocketService,
}

#[derive(Debug)]
/// Available message types to process
pub enum Message {
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
        // Create the websocket service
        let callback = link.send_back(|data| Message::LoginResponse(data));
        let notification = link.send_back(|_| Message::WebSocketIgnore);

        // Create the component
        Self {
            username: String::new(),
            password: String::new(),
            button_disabled: true,
            cookie_service: CookieService::new(),
            console_service: ConsoleService::new(),
            websocket_service: WebSocketService::new_with_callbacks(callback, notification)
                .expect("No valid websocket connection"),
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
                .write_login_credential_request(&self.username, &self.password)
            {
                Ok(data) => {
                    // Remove the current session cookie
                    self.cookie_service.remove(SESSION_COOKIE);

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
            Message::LoginResponse(mut response) => match self.protocol_service.read_login_response(&mut response) {
                Ok(token) => {
                    self.console_service.info("Login succeed");

                    // Set the retrieved session cookie
                    self.cookie_service.set(SESSION_COOKIE, &token);

                    true
                }
                Err(e) => {
                    self.console_service
                        .error(&format!("Unable to succeed with login response: {}", e));
                    false
                }
            },
            Message::UpdateUsername(new_username) => {
                self.username = new_username;
                self.button_disabled = self.username.is_empty() || self.password.is_empty();
                true
            }
            Message::UpdatePassword(new_password) => {
                self.password = new_password;
                self.button_disabled = self.username.is_empty() || self.password.is_empty();
                true
            }
            _ => false,
        }
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
