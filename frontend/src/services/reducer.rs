//! The message reducer agent

use serde_cbor::from_slice;
use services::websocket::{WebSocketService, WebSocketResponse};
use std::collections::{HashMap, HashSet};
use webapp::protocol::Response;
use yew::prelude::worker::*;

#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ResponseType {
    StatusClose,
    StatusOpen,
    StatusError,
    LoginSession,
    LoginCredentials,
    Logout,
    Error,
}

#[derive(Deserialize, Serialize)]
/// Available reducer requests
pub enum ReducerRequest {
    Subscribe(Vec<ResponseType>),
    Send(Vec<u8>),
}

impl Transferable for ReducerRequest {}

#[derive(Clone, Deserialize, Serialize)]
/// Available reducer requests
pub enum ReducerResponse {
    Close,
    Error,
    Open,
    Data(Response),
}

impl Transferable for ReducerResponse {}

/// The ReducerAgent which filters websocket data and handles its connection
pub struct ReducerAgent {
    link: AgentLink<ReducerAgent>,
    subscribers: HashMap<HandlerId, HashSet<ResponseType>>,
    websocket_service: WebSocketService,
}

impl ReducerAgent {
    fn respond_data_filtered(&self, response_type: &ResponseType, response: Response) {
        self.respond_filtered(response_type, &ReducerResponse::Data(response));
    }

    fn respond_filtered(&self, response_type: &ResponseType, response: &ReducerResponse) {
        for (who, responses) in &self.subscribers {
            if responses.contains(response_type) {
                self.link.response(*who, response.clone());
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
        let websocket_service = WebSocketService::new(link.send_back(|r| r));
        Self {
            link,
            subscribers: HashMap::new(),
            websocket_service,
        }
    }

    /// Internal update mechanism based on messages
    fn update(&mut self, msg: Self::Message) {
        match msg {
            WebSocketResponse::Data(data) => match from_slice(&data) {
                Ok(r @ Response::LoginSession(_)) => self.respond_data_filtered(&ResponseType::LoginSession, r),
                Ok(r @ Response::LoginCredentials(_)) => self.respond_data_filtered(&ResponseType::LoginCredentials, r),
                Ok(r @ Response::Logout(_)) => self.respond_data_filtered(&ResponseType::Logout, r),
                Ok(r @ Response::Error) => self.respond_data_filtered(&ResponseType::Error, r),
                Err(_) => {} // Message not decodable
            },
            // Inform all subscribers about the status responses
            WebSocketResponse::Close => self.respond_filtered(&ResponseType::StatusClose, &ReducerResponse::Close),
            WebSocketResponse::Error => self.respond_filtered(&ResponseType::StatusError, &ReducerResponse::Error),
            WebSocketResponse::Open => self.respond_filtered(&ResponseType::StatusOpen, &ReducerResponse::Open),
        }
    }

    /// Handle incoming data requests
    fn handle(&mut self, msg: Self::Input, id: HandlerId) {
        match msg {
            ReducerRequest::Subscribe(items) => for i in items {
                self.subscribers
                    .entry(id)
                    .and_modify(|e| {
                        e.insert(i.clone());
                    })
                    .or_insert_with(|| vec![i].into_iter().collect());
            },
            ReducerRequest::Send(data) => self.websocket_service.send(&data),
        }
    }

    /// Add a client to the pool of connections of this agent
    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id, HashSet::new());
    }

    /// Remove a client from the pool of connections of this agent
    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}
