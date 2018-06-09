//! The Login component
use failure::Error;
use yew::format::Json;
use yew::prelude::*;
use yew::services::console::ConsoleService;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use yew::services::Task;

/// Data Model for the Login component
pub struct LoginComponent {
    username: String,
    password: String,
    websocket_task: Option<WebSocketTask>,
}

#[derive(Debug)]
pub enum Msg {
    Login,
    UpdateUsername(String),
    UpdatePassword(String),
    WebSocketReady(Result<WsResponse, Error>),
    WebSocketAction(WsAction),
    WebSocketIgnore,
    WebSocketError,
}

impl From<WsAction> for Msg {
    fn from(action: WsAction) -> Self {
        Msg::WebSocketAction(action)
    }
}

/// This type is used as a request which sent to websocket connection.
#[derive(Serialize, Debug)]
struct WsRequest {
    value: u32,
}

/// This type is an expected response from a websocket connection.
#[derive(Deserialize, Debug)]
pub struct WsResponse {
    value: u32,
}

/// WebSocketAction representation
#[derive(Debug)]
pub enum WsAction {
    SendData,
    Disconnect,
    Lost,
}

impl<C> Component<C> for LoginComponent
where
    C: 'static + AsMut<ConsoleService> + AsMut<WebSocketService>,
{
    type Message = Msg;
    type Properties = ();

    fn create(_: (), _: &mut Env<C, Self>) -> Self {
        LoginComponent {
            username: String::new(),
            password: String::new(),
            websocket_task: None,
        }
    }

    fn update(&mut self, msg: Self::Message, ctx: &mut Env<C, Self>) -> ShouldRender {
        {
            let console: &mut ConsoleService = ctx.as_mut();
            console.log(&format!("Message: {:?}", msg));
        }
        match msg {
            Msg::Login => {
                let callback = ctx.send_back(|Json(data)| Msg::WebSocketReady(data));
                let notification = ctx.send_back(|status| match status {
                    WebSocketStatus::Opened => Msg::WebSocketIgnore,
                    WebSocketStatus::Closed => WsAction::Lost.into(),
                    WebSocketStatus::Error => Msg::WebSocketError,
                });
                let websocket: &mut WebSocketService = ctx.as_mut();
                let task = websocket.connect("ws://localhost:9001/", callback, notification);
                self.websocket_task = Some(task);
            }
            Msg::UpdateUsername(new_username) => {
                self.username = new_username;
            }
            Msg::UpdatePassword(new_password) => {
                self.password = new_password;
            }
            Msg::WebSocketAction(action) => match action {
                WsAction::SendData => {
                    let request = WsRequest { value: 321 };
                    if let Some(ws) = self.websocket_task.as_mut() {
                        ws.send(Json(&request));
                    }
                }
                WsAction::Disconnect => {
                    if let Some(mut ws) = self.websocket_task.take() {
                        ws.cancel();
                    }
                }
                WsAction::Lost => {
                    self.websocket_task = None;
                }
            },
            Msg::WebSocketReady(response) => {}
            Msg::WebSocketIgnore => {}
            Msg::WebSocketError => {
                let console: &mut ConsoleService = ctx.as_mut();
                console.error("Websocket error");
            }
        };
        true
    }
}

impl<C> Renderable<C, LoginComponent> for LoginComponent
where
    C: 'static + AsMut<ConsoleService> + AsMut<WebSocketService>,
{
    fn view(&self) -> Html<C, Self> {
        html! {
            <form onsubmit="return false", />
                <input value=&self.username, oninput=|e| Msg::UpdateUsername(e.value), />
                <input type="password", value=&self.password, oninput=|e| Msg::UpdatePassword(e.value), />
                <button type="submit", onclick=|_| Msg::Login,>{"Login"}</button>
            </form>
        }
    }
}
