//! The SessionTimer agent

use std::time::Duration;
use yew::{
    prelude::worker::*,
    prelude::*,
    services::{ConsoleService, IntervalService, Task},
};

/// A simple update message
pub struct Update;

#[derive(Deserialize, Serialize)]
/// Available timer requests
pub enum Request {
    Start,
    Stop,
}

impl Transferable for Request {}

#[derive(Deserialize, Serialize)]
/// Available reducer requests
pub struct Response;

impl Transferable for Response {}

pub struct SessionTimerAgent {
    callback: Callback<()>,
    console_service: ConsoleService,
    timer_task: Option<Box<Task>>,
}

impl Agent for SessionTimerAgent {
    type Reach = Context;
    type Message = Update;
    type Input = Request;
    type Output = Response;

    /// Creates a new SessionTimerAgent
    fn create(link: AgentLink<Self>) -> Self {
        Self {
            callback: link.send_back(|_| Update),
            console_service: ConsoleService::new(),
            timer_task: None,
        }
    }

    /// Internal update mechanism based on messages
    fn update(&mut self, _: Self::Message) {
        self.console_service.log("Update");
    }

    /// Handle incoming data requests
    fn handle(&mut self, msg: Self::Input, _: HandlerId) {
        match msg {
            Request::Start => {
                let handle = IntervalService::new().spawn(Duration::from_secs(1), self.callback.clone());
                self.timer_task = Some(Box::new(handle));
            }
            Request::Stop => if let Some(mut task) = self.timer_task.take() {
                task.cancel();
                self.timer_task = None;
            },
        }
    }
}
