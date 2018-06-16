//! The Login component

use frontend::services::{protocol::ProtocolService, websocket::WebSocketService};
use yew::{prelude::*, services::console::ConsoleService};

/// Data Model for the Login component
pub struct LoginComponent {
    username: String,
    password: String,
    websocket_service: WebSocketService,
    protocol_service: ProtocolService,
}

#[derive(Debug)]
pub enum Msg {
    LoginRequest,
    LoginResponse(Vec<u8>),
    WebSocketIgnore,
    UpdateUsername(String),
    UpdatePassword(String),
}

impl<C> Component<C> for LoginComponent
where
    C: 'static + AsMut<ConsoleService>,
{
    type Message = Msg;
    type Properties = ();

    fn create(_: (), env: &mut Env<C, Self>) -> Self {
        // Setup the websocket connection
        let callback = env.send_back(|data| Msg::LoginResponse(data));
        let notification = env.send_back(|_| Msg::WebSocketIgnore);

        // Create the websocket service
        let websocket_service = WebSocketService::new(callback, notification).expect("No valid websocket connection");

        // Create the protocol service
        let protocol_service = ProtocolService::new();

        LoginComponent {
            username: String::new(),
            password: String::new(),
            websocket_service,
            protocol_service,
        }
    }

    fn update(&mut self, msg: Self::Message, ctx: &mut Env<C, Self>) -> ShouldRender {
        match msg {
            Msg::LoginRequest => match self
                .protocol_service
                .write_login_request(&self.username, &self.password)
            {
                Err(e) => ctx.as_mut().error(&format!("Unable to create login request: {}", e)),
                Ok(data) => self.websocket_service.send(data),
            },
            Msg::LoginResponse(mut response) => {
                let console: &mut ConsoleService = ctx.as_mut();
                match self.protocol_service.read_login_response(&mut response) {
                    Err(e) => console.error(&format!("Unable to read login response: {}", e)),
                    Ok(success) => console.info(&format!("Login succeed: {}", success)),
                }
            }
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

impl<C> Renderable<C, LoginComponent> for LoginComponent
where
    C: 'static + AsMut<ConsoleService>,
{
    fn view(&self) -> Html<C, Self> {
        html! {
            <form class=("uk-container", "uk-container-small"), onsubmit="return false",>
                <fieldset class="uk-fieldset",>
                    <legend class="uk-legend",>{"Authentication Demo"}</legend>
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
        }
    }
}
