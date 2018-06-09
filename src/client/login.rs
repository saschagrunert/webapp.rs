//! The Login component
use yew::prelude::*;
use yew::services::console::ConsoleService;

/// Data Model for the Login component
pub struct LoginComponent {
    username: String,
    password: String,
}

#[derive(Debug)]
pub enum Msg {
    Login,
    UpdateUsername(String),
    UpdatePassword(String),
}

impl<C> Component<C> for LoginComponent
where
    C: 'static + AsMut<ConsoleService>,
{
    type Message = Msg;
    type Properties = ();

    fn create(_: (), _: &mut Env<C, Self>) -> Self {
        LoginComponent {
            username: String::new(),
            password: String::new(),
        }
    }

    fn update(&mut self, msg: Self::Message, ctx: &mut Env<C, Self>) -> ShouldRender {
        let console: &mut ConsoleService = ctx.as_mut();
        console.log(&format!("Message: {:?}", msg));
        match msg {
            Msg::Login => {}
            Msg::UpdateUsername(new_username) => {
                self.username = new_username;
            }
            Msg::UpdatePassword(new_password) => {
                self.password = new_password;
            }
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
            <form onsubmit="return false", />
                <input value=&self.username, oninput=|e| Msg::UpdateUsername(e.value), />
                <input type="password", value=&self.password, oninput=|e| Msg::UpdatePassword(e.value), />
                <button type="submit", onclick=|_| Msg::Login,>{"Login"}</button>
            </form>
        }
    }
}
