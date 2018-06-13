//! The Root component
use frontend::login::LoginComponent;
use yew::prelude::*;
use yew::services::console::ConsoleService;
use yew::services::websocket::WebSocketService;

/// The main context of the application
pub struct Context {
    /// The console which can be logged
    pub console: ConsoleService,

    /// The websocket which will be used for server communication
    pub websocket: WebSocketService,
}

impl AsMut<ConsoleService> for Context {
    fn as_mut(&mut self) -> &mut ConsoleService {
        &mut self.console
    }
}

impl AsMut<WebSocketService> for Context {
    fn as_mut(&mut self) -> &mut WebSocketService {
        &mut self.websocket
    }
}

/// Data Model for the Root Component
pub struct RootComponent {}

impl<C> Component<C> for RootComponent
where
    C: AsMut<ConsoleService> + AsMut<WebSocketService>,
{
    type Message = ();
    type Properties = ();

    fn create(_: (), _: &mut Env<C, Self>) -> Self {
        RootComponent {}
    }

    fn update(&mut self, _: Self::Message, _: &mut Env<C, Self>) -> ShouldRender {
        true
    }
}

impl<C> Renderable<C, RootComponent> for RootComponent
where
    C: 'static + AsMut<ConsoleService> + AsMut<WebSocketService>,
{
    fn view(&self) -> Html<C, Self> {
        html! {
            <LoginComponent:/>
        }
    }
}
