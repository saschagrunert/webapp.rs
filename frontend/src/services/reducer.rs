//! The message reducer agent

use serde_cbor::from_slice;
use services::websocket::{WebSocketAgent, WebSocketRequest, WebSocketResponse};
use std::collections::{HashMap, HashSet};
use webapp::protocol::Response;
use yew::prelude::worker::*;

#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ResponseType {
    LoginSession,
    LoginCredentials,
    Logout,
    Error,
}

#[derive(Deserialize, Serialize)]
/// Available reducer requests
pub enum ReducerRequest {
    Subscribe(ResponseType),
    Send(Vec<u8>),
}

impl Transferable for ReducerRequest {}

#[derive(Clone, Deserialize, Serialize)]
/// Available reducer requests
pub enum ReducerResponse {
    Closed,
    Error,
    Opened,
    Data(Response),
}

impl Transferable for ReducerResponse {}

/// The ReducerAgent which filters websocket data and handles its connection
pub struct ReducerAgent {
    link: AgentLink<ReducerAgent>,
    subscribers: HashMap<HandlerId, HashSet<ResponseType>>,
    websocket_agent: Box<Bridge<WebSocketAgent>>,
}

impl ReducerAgent {
    fn respond(&self, response: &ReducerResponse) {
        for who in self.subscribers.keys() {
            self.link.response(*who, response.clone());
        }
    }

    fn filter_respond(&self, response_type: &ResponseType, response: &Response) {
        for (who, responses) in &self.subscribers {
            if responses.contains(response_type) {
                self.link.response(*who, ReducerResponse::Data(response.clone()));
            }
        }
    }
}

impl Agent for ReducerAgent {
    type Reach = Context;
    type Message = WebSocketResponse;
    type Input = ReducerRequest;
    type Output = ReducerResponse;

    /// Creates a new ReducerAgent
    fn create(link: AgentLink<Self>) -> Self {
        let websocket_agent = WebSocketAgent::bridge(link.send_back(|r| r));

        // Return the instance
        Self {
            link,
            subscribers: HashMap::new(),
            websocket_agent,
        }
    }

    /// Internal update mechanism based on messages
    fn update(&mut self, msg: Self::Message) {
        match msg {
            WebSocketResponse::Data(data) => match from_slice(&data) {
                Ok(r @ Response::LoginSession(_)) => self.filter_respond(&ResponseType::LoginSession, &r),
                Ok(r @ Response::LoginCredentials(_)) => self.filter_respond(&ResponseType::LoginCredentials, &r),
                Ok(r @ Response::Logout(_)) => self.filter_respond(&ResponseType::Logout, &r),
                Ok(r @ Response::Error) => self.filter_respond(&ResponseType::Error, &r),
                Err(_) => {} // Message not decodable
            },
            // Inform all subscribers about the status responses
            WebSocketResponse::Closed => self.respond(&ReducerResponse::Closed),
            WebSocketResponse::Error => self.respond(&ReducerResponse::Error),
            WebSocketResponse::Opened => self.respond(&ReducerResponse::Opened),
        }
    }

    /// Handle incoming data requests
    fn handle(&mut self, msg: Self::Input, id: HandlerId) {
        match msg {
            ReducerRequest::Subscribe(t) => {
                self.subscribers
                    .entry(id)
                    .and_modify(|e| {
                        e.insert(t.clone());
                    })
                    .or_insert_with(|| vec![t].into_iter().collect());
            }
            ReducerRequest::Send(data) => self.websocket_agent.send(WebSocketRequest(data)),
        }
    }

    /// Remove a client from the pool of connections of this agent
    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}
