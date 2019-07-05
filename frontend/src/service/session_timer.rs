//! The SessionTimer agent

use crate::{api::Response, service::cookie::CookieService, SESSION_COOKIE};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use webapp::{
    protocol::{model::Session, request::LoginSession, response::Login},
    API_URL_LOGIN_SESSION,
};
use yew::{
    format::Cbor,
    prelude::{worker::*, *},
    services::{fetch::FetchTask, IntervalService, Task},
};

/// Possible message types
pub enum Message {
    Fetch(Response<Login>),
    Update,
}

#[derive(Deserialize, Serialize)]
/// Available timer requests
pub enum Request {
    Start,
    Stop,
}

impl Transferable for Request {}

#[derive(Deserialize, Serialize)]
pub struct TimerResponse;

impl Transferable for TimerResponse {}

pub struct SessionTimerAgent {
    agent_link: AgentLink<SessionTimerAgent>,
    callback: Callback<()>,
    cookie_service: CookieService,
    fetch_task: Option<FetchTask>,
    timer_task: Option<Box<Task>>,
}

impl Agent for SessionTimerAgent {
    type Input = Request;
    type Message = Message;
    type Output = TimerResponse;
    type Reach = Context;

    /// Creates a new SessionTimerAgent
    fn create(link: AgentLink<Self>) -> Self {
        Self {
            callback: link.send_back(|_| Message::Update),
            agent_link: link,
            cookie_service: CookieService::new(),
            fetch_task: None,
            timer_task: None,
        }
    }

    /// Internal update mechanism based on messages
    fn update(&mut self, msg: Self::Message) {
        match msg {
            Message::Update => {
                info!("Updating current session");
                if let Ok(token) = self.cookie_service.get(SESSION_COOKIE) {
                    self.fetch_task = fetch! {
                        LoginSession(Session::new(token)) => API_URL_LOGIN_SESSION,
                        self.agent_link, Message::Fetch,
                        || {},
                        || {
                            warn!("Unable to create scheduled session login request");
                        }
                    };
                }
            }
            Message::Fetch(response) => {
                let (meta, Cbor(body)) = response.into_parts();

                // Check the response type
                if meta.status.is_success() {
                    match body {
                        Ok(Login(Session { token })) => {
                            info!("Scheduled session based login succeed");

                            // Set the retrieved session cookie
                            self.cookie_service.set(SESSION_COOKIE, &token);
                        }
                        _ => warn!("Got wrong scheduled session login response"),
                    }
                } else {
                    // Authentication failed
                    info!(
                        "Scheduled session login failed with status: {}",
                        meta.status
                    );
                }

                // Remove the ongoing task
                self.fetch_task = None;
            }
        }
    }

    /// Handle incoming data requests
    fn handle(&mut self, msg: Self::Input, _: HandlerId) {
        match msg {
            Request::Start => {
                let handle =
                    IntervalService::new().spawn(Duration::from_secs(10), self.callback.clone());
                self.timer_task = Some(Box::new(handle));
            }
            Request::Stop => {
                if let Some(mut task) = self.timer_task.take() {
                    task.cancel();
                    self.timer_task = None;
                }
            }
        }
    }
}
