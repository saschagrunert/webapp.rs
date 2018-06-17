//! The Login component

use frontend::services::{cookie::CookieService, protocol::ProtocolService, websocket::WebSocketService};
use yew::{prelude::*, services::ConsoleService};

const COOKIE_NAME: &str = "sessionToken";

/// Data Model for the Login component
pub struct LoginComponent {
    username: String,
    password: String,
    cookie_service: CookieService,
    console_service: ConsoleService,
    protocol_service: ProtocolService,
    websocket_service: WebSocketService,
}

#[derive(Debug)]
pub enum Message {
    LoginCredentialRequest,
    LoginTokenRequest(String),
    LoginResponse(Vec<u8>),
    LogoutRequest,
    UpdateUsername(String),
    UpdatePassword(String),
    WebSocketIgnore,
}

impl Component for LoginComponent {
    type Message = Message;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        // Setup the websocket connection
        let callback = link.send_back(|data| Message::LoginResponse(data));
        let notification = link.send_back(|_| Message::WebSocketIgnore);

        // Create the websocket service
        let websocket_service = WebSocketService::new(callback, notification).expect("No valid websocket connection");

        // Create the protocol service
        let protocol_service = ProtocolService::new();

        // Create the component
        LoginComponent {
            username: String::new(),
            password: String::new(),
            cookie_service: CookieService::new(),
            console_service: ConsoleService::new(),
            websocket_service,
            protocol_service,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::LoginTokenRequest(token) => match self.protocol_service.write_login_token_request(&token) {
                Ok(data) => {
                    // Send the request
                    self.websocket_service.send(data);
                    false
                }
                Err(e) => {
                    self.console_service
                        .error(&format!("Unable to create login token request: {}", e));
                    false
                }
            },
            Message::LoginCredentialRequest => match self
                .protocol_service
                .write_login_credential_request(&self.username, &self.password)
            {
                Ok(data) => {
                    // Remove the current session cookie
                    self.cookie_service.remove_cookie(COOKIE_NAME);

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
                    self.console_service.info(&format!("Login succeed: {}", token));

                    // Set the retrieved session cookie
                    self.cookie_service.set_cookie(COOKIE_NAME, &token, 365);

                    true
                }
                Err(e) => {
                    self.console_service
                        .error(&format!("Unable to succeed with login response: {}", e));
                    false
                }
            },
            Message::LogoutRequest => {
                // Just remove the cookie for now and update the user interface
                self.cookie_service.remove_cookie(COOKIE_NAME);
                true
            }
            Message::UpdateUsername(new_username) => {
                self.username = new_username;
                false
            }
            Message::UpdatePassword(new_password) => {
                self.password = new_password;
                false
            }
            _ => false,
        }
    }
}

impl Renderable<LoginComponent> for LoginComponent {
    fn view(&self) -> Html<Self> {
        match self.cookie_service.get_cookie(COOKIE_NAME) {
            Ok(_) => html! {
                <div class=("uk-card", "uk-card-default", "uk-card-body",
                            "uk-width-1-3@s", "uk-position-center"),>
                    <button class=("uk-button", "uk-button-default"),
                            onclick=|_| Message::LogoutRequest,>{"Logout"}</button>
                </div>
            },
            _ => html! {
                <div class=("uk-card", "uk-card-default", "uk-card-body",
                            "uk-width-1-3@s", "uk-position-center"),>
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
                            <button class=("uk-button", "uk-button-default"),
                                    type="submit",
                                    onclick=|_| Message::LoginCredentialRequest,>{"Login"}</button>
                        </fieldset>
                    </form>
                </div>
            },
        }
    }
}
