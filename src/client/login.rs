//! The Login component
use yew::prelude::*;
use yew::services::console::ConsoleService;

/// Data Model for the Login component
pub struct LoginComponent {}

#[derive(Debug)]
pub enum Msg {
    Login,
}

impl<C> Component<C> for LoginComponent
where
    C: AsMut<ConsoleService>,
{
    type Message = Msg;
    type Properties = ();

    fn create(_: (), _: &mut Env<C, Self>) -> Self {
        LoginComponent {}
    }

    fn update(&mut self, msg: Self::Message, ctx: &mut Env<C, Self>) -> ShouldRender {
        ctx.as_mut().log(&format!("Message: {:?}", msg));
        true
    }
}

impl<C> Renderable<C, LoginComponent> for LoginComponent
where
    C: 'static + AsMut<ConsoleService>,
{
    fn view(&self) -> Html<C, Self> {
        html! {
            <button onclick=|_| Msg::Login,>{"Login"}</button>
        }
    }
}
