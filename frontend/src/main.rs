use anyhow::Result;
use webapp_frontend::{init_logger, RootComponent};
use yew::prelude::*;

fn main() -> Result<()> {
    // Initialize the application
    yew::initialize();

    // Initialize the logger
    init_logger()?;

    // Create a new app
    App::<RootComponent>::new().mount_to_body();

    // Run the application for development purposes
    yew::run_loop();

    Ok(())
}
