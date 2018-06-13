//! The Login component
use failure::Error;
use frontend::services::websocket::{WebSocketService, WebSocketTask};
use shared::{LoginRequestData, WsMessage};
use yew::format::Cbor;
use yew::prelude::*;
use yew::services::console::ConsoleService;
use API_URL;

/// Data Model for the Login component
pub struct LoginComponent {
    request: LoginRequestData,
    web_socket_task: WebSocketTask,
}

#[derive(Debug)]
pub enum Msg {
    LoginRequest,
    LoginResponse(Result<WsMessage, Error>),
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
        let callback = env.send_back(|Cbor(data)| Msg::LoginResponse(data));
        let notification = env.send_back(|_| Msg::WebSocketIgnore);
        let mut service = WebSocketService::new();

        // Create the websocket task
        let task = service.connect(API_URL, callback, notification);

        LoginComponent {
            request: LoginRequestData {
                username: String::new(),
                password: String::new(),
            },
            web_socket_task: task,
        }
    }

    fn update(&mut self, msg: Self::Message, ctx: &mut Env<C, Self>) -> ShouldRender {
        match msg {
            Msg::LoginRequest => {
                let msg = WsMessage::LoginRequest(self.request.clone());
                self.web_socket_task.send(Cbor(&msg));
            }
            Msg::LoginResponse(response) => {
                let console: &mut ConsoleService = ctx.as_mut();
                match response {
                    Err(e) => console.error(&format!("Error: {}", e)),
                    Ok(d) => console.log(&format!("Response: {:?}", d)),
                }
            }
            Msg::UpdateUsername(new_username) => {
                self.request.username = new_username;
            }
            Msg::UpdatePassword(new_password) => {
                self.request.password = new_password;
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
                               value=&self.request.username,
                               oninput=|e| Msg::UpdateUsername(e.value), />
                    </div>
                    <div class="uk-margin",>
                        <input class="uk-input",
                               type="password",
                               placeholder="Password",
                               value=&self.request.password,
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
