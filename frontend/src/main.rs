extern crate webapp_frontend;
extern crate yew;

use webapp_frontend::RootComponent;
use yew::prelude::*;

fn main() {
    // Initialize the application
    yew::initialize();

    // Create a new app
    App::<RootComponent>::new().mount_to_body();

    // Run the application for development purposes
    yew::run_loop();
}
