//! The Login component
use capnp::{message::Builder, serialize_packed::write_message};
use failure::Error;
use frontend::services::websocket::{WebSocketService, WebSocketTask};
use protocol_capnp::request;
use yew::{prelude::*, services::console::ConsoleService};
use API_URL;

/// Data Model for the Login component
pub struct LoginComponent {
    username: String,
    password: String,
    web_socket_task: WebSocketTask,
}

#[derive(Debug)]
pub enum Msg {
    LoginRequest,
    LoginResponse(Result<Vec<u8>, Error>),
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
        let callback = env.send_back(|data: Result<Vec<u8>, Error>| Msg::LoginResponse(data));
        let notification = env.send_back(|_| Msg::WebSocketIgnore);
        let mut service = WebSocketService::new();

        // Create the websocket task
        let task = service.connect(API_URL, callback, notification);

        LoginComponent {
            username: String::new(),
            password: String::new(),
            web_socket_task: task,
        }
    }

    fn update(&mut self, msg: Self::Message, ctx: &mut Env<C, Self>) -> ShouldRender {
        match msg {
            Msg::LoginRequest => {
                // Create an empty message
                let mut message = Builder::new_default();
                {
                    // Set the request parameters
                    let request = message.init_root::<request::Builder>();
                    let mut login = request.init_login();
                    login.set_username(&self.username);;
                    login.set_password(&self.password);;
                }

                // Serialize to a vector
                let mut data = Vec::new();
                write_message(&mut data, &message).unwrap();

                // Send the data
                self.web_socket_task.send(&data);
            }
            Msg::LoginResponse(response) => {
                let console: &mut ConsoleService = ctx.as_mut();
                match response {
                    Err(e) => console.error(&format!("Error: {}", e)),
                    Ok(d) => console.log(&format!("Response: {:?}", d)),
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
