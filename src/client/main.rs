extern crate webapp;
extern crate yew;

use webapp::{Context, RootComponent};
use yew::prelude::*;
use yew::services::console::ConsoleService;
use yew::services::websocket::WebSocketService;

fn main() {
    // Initialize the application
    yew::initialize();
    let context = Context {
        console: ConsoleService::new(),
        websocket: WebSocketService::new(),
    };
    let app: App<_, RootComponent> = App::new(context);

    // Set the root component
    app.mount_to_body();

    // Run the application for development purposes
    yew::run_loop();
}
