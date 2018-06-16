//! The Root component
use frontend::login::LoginComponent;
use yew::prelude::*;
use yew::services::console::ConsoleService;

/// The main context of the application
pub struct Context {
    /// The console which can be logged
    pub console: ConsoleService,
}

impl AsMut<ConsoleService> for Context {
    fn as_mut(&mut self) -> &mut ConsoleService {
        &mut self.console
    }
}

/// Data Model for the Root Component
pub struct RootComponent {}

impl Component for RootComponent {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        RootComponent {}
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }
}

impl Renderable<RootComponent> for RootComponent {
    fn view(&self) -> Html<Self> {
        html! {
            <LoginComponent:/>
        }
    }
}
