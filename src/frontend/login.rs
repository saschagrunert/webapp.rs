//! The Login component

use frontend::services::{protocol::ProtocolService, websocket::WebSocketService};
use yew::{prelude::*, services::ConsoleService};

/// Data Model for the Login component
pub struct LoginComponent {
    username: String,
    password: String,
    console_service: ConsoleService,
    protocol_service: ProtocolService,
    websocket_service: WebSocketService,
}

#[derive(Debug)]
pub enum Msg {
    LoginRequest,
    LoginResponse(Vec<u8>),
    WebSocketIgnore,
    UpdateUsername(String),
    UpdatePassword(String),
}

impl Component for LoginComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        // Setup the websocket connection
        let callback = link.send_back(|data| Msg::LoginResponse(data));
        let notification = link.send_back(|_| Msg::WebSocketIgnore);

        // Create the websocket service
        let websocket_service = WebSocketService::new(callback, notification).expect("No valid websocket connection");

        // Create the protocol service
        let protocol_service = ProtocolService::new();

        LoginComponent {
            username: String::new(),
            password: String::new(),
            console_service: ConsoleService::new(),
            websocket_service,
            protocol_service,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::LoginRequest => match self
                .protocol_service
                .write_login_request(&self.username, &self.password)
            {
                Err(e) => self
                    .console_service
                    .error(&format!("Unable to create login request: {}", e)),
                Ok(data) => self.websocket_service.send(data),
            },
            Msg::LoginResponse(mut response) => match self.protocol_service.read_login_response(&mut response) {
                Err(e) => self
                    .console_service
                    .error(&format!("Unable to read login response: {}", e)),
                Ok(success) => self.console_service.info(&format!("Login succeed: {}", success)),
            },
            Msg::UpdateUsername(new_username) => {
                self.username = new_username;
            }
            Msg::UpdatePassword(new_password) => {
                self.password = new_password;
            }
            _ => {}
        };
        true
    }
}

impl Renderable<LoginComponent> for LoginComponent {
    fn view(&self) -> Html<Self> {
        html! {
            <div class=("uk-card", "uk-card-default", "uk-card-body",
                        "uk-width-1-3@s", "uk-position-center"),>
                <form onsubmit="return false",>
                    <fieldset class="uk-fieldset",>
                        <legend class="uk-legend",>{"Authentication"}</legend>
                        <div class="uk-margin",>
                            <input class="uk-input",
                                placeholder="Username",
                                value=&self.username,
                                oninput=|e| Msg::UpdateUsername(e.value), />
                        </div>
                        <div class="uk-margin",>
                            <input class="uk-input",
                                type="password",
                                placeholder="Password",
                                value=&self.password,
                                oninput=|e| Msg::UpdatePassword(e.value), />
                        </div>
                        <button class=("uk-button", "uk-button-default"),
                                type="submit",
                                onclick=|_| Msg::LoginRequest,>{"Login"}</button>
                    </fieldset>
                </form>
            </div>
        }
    }
}
