extern crate webapp;
extern crate yew;

use webapp::RootComponent;
use yew::prelude::*;

fn main() {
    // Initialize the application
    yew::initialize();

    // Create a new app
    App::<RootComponent>::new().mount_to_body();

    // Run the application for development purposes
    yew::run_loop();
}
