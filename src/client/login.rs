//! The Login component
use yew::prelude::*;
use yew::services::console::ConsoleService;

/// Data Model for the Login component
pub struct LoginComponent {
    name: String,
}

#[derive(Debug)]
pub enum Msg {
    Login,
    UpdateName(String),
}

impl<C> Component<C> for LoginComponent
where
    C: AsMut<ConsoleService>,
{
    type Message = Msg;
    type Properties = ();

    fn create(_: (), _: &mut Env<C, Self>) -> Self {
        LoginComponent { name: String::new() }
    }

    fn update(&mut self, msg: Self::Message, ctx: &mut Env<C, Self>) -> ShouldRender {
        ctx.as_mut().log(&format!("Message: {:?}", msg));
        match msg {
            Msg::Login => false,
            Msg::UpdateName(new_name) => {
                self.name = new_name;
                true
            }
        }
    }
}

impl<C> Renderable<C, LoginComponent> for LoginComponent
where
    C: 'static + AsMut<ConsoleService>,
{
    fn view(&self) -> Html<C, Self> {
        html! {
            <p>{ self.name.chars().rev().collect::<String>() }</p>
            <button onclick=|_| Msg::Login,>{"Login"}</button>
            <input value=&self.name, oninput=|e| Msg::UpdateName(e.value), />
        }
    }
}
