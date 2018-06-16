extern crate webapp;
extern crate yew;

use webapp::RootComponent;
use yew::prelude::*;

fn main() {
    // Initialize the application
    yew::initialize();
    let app: App<RootComponent> = App::new();

    // Set the root component
    app.mount_to_body();

    // Run the application for development purposes
    yew::run_loop();
}
