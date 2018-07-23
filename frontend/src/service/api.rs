//! The API agent

use failure::Error;
use std::collections::{HashMap, HashSet};
use webapp::protocol::{response::Login, Request, Response};
use yew::{
    format::Cbor,
    prelude::worker::*,
    services::{
        fetch::{FetchTask, Request as FetchRequest, Response as FetchResponse},
        ConsoleService, FetchService, Task,
    },
};

#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ResponseType {
    Error,
    LoginCredentials,
    LoginSession,
    Logout,
}

pub struct Message(FetchResponse<Cbor<Result<Response, Error>>>);

#[derive(Deserialize, Serialize)]
/// Available reducer requests
pub enum ApiRequest {
    Send(Request),
    Subscribe(Vec<ResponseType>),
}

impl Transferable for ApiRequest {}

#[derive(Clone, Deserialize, Serialize)]
/// Available reducer requests
pub enum ApiResponse {
    Data(Response),
    RequestEncodeFailed,
    ResponseDecodeFailed,
}

impl Transferable for ApiResponse {}

pub struct ApiAgent {
    console_service: ConsoleService,
    fetch_service: FetchService,
    fetch_tasks: Vec<FetchTask>,
    link: AgentLink<ApiAgent>,
    subscribers: HashMap<HandlerId, HashSet<ResponseType>>,
}

impl ApiAgent {
    fn respond_data_filtered(&self, response_type: &ResponseType, response: Response) {
        self.respond_filtered(response_type, &ApiResponse::Data(response));
    }

    fn respond_filtered(&self, response_type: &ResponseType, response: &ApiResponse) {
        for (who, responses) in &self.subscribers {
            if responses.contains(response_type) {
                self.link.response(*who, response.clone());
            }
        }
    }

    fn cleanup_tasks(&mut self) {
        self.fetch_tasks.retain(|e| e.is_active());
    }
}

impl Agent for ApiAgent {
    type Reach = Context;
    type Message = Message;
    type Input = ApiRequest;
    type Output = ApiResponse;

    /// Creates a new ApiAgent
    fn create(link: AgentLink<Self>) -> Self {
        Self {
            console_service: ConsoleService::new(),
            fetch_service: FetchService::new(),
            fetch_tasks: Vec::new(),
            link,
            subscribers: HashMap::new(),
        }
    }

    /// Internal update mechanism based on messages
    fn update(&mut self, msg: Self::Message) {
        let Message(response) = msg;
        let (meta, Cbor(body)) = response.into_parts();

        self.console_service.info(&format!("Meta: {:?}", meta));
        self.console_service.info(&format!("Body: {:?}", body));

        if meta.status.is_success() {
            match body {
                Ok(r @ Response::Login(Login::Session(_))) => {
                    self.respond_data_filtered(&ResponseType::LoginSession, r)
                }
                Ok(r @ Response::Login(Login::Credentials(_))) => {
                    self.respond_data_filtered(&ResponseType::LoginCredentials, r)
                }
                Ok(r @ Response::Logout(_)) => self.respond_data_filtered(&ResponseType::Logout, r),
                Ok(r @ Response::Error) => self.respond_data_filtered(&ResponseType::Error, r),
                Err(e) => {
                    self.console_service.warn(&format!("Response not decodable: {}", e));
                    self.respond_filtered(&ResponseType::Error, &ApiResponse::ResponseDecodeFailed);
                }
            }
        } else {
            // Do something else
        }
    }

    /// Handle incoming data requests
    fn handle(&mut self, msg: Self::Input, id: HandlerId) {
        match msg {
            ApiRequest::Subscribe(items) => for i in items {
                self.subscribers
                    .entry(id)
                    .and_modify(|e| {
                        e.insert(i.clone());
                    })
                    .or_insert_with(|| vec![i].into_iter().collect());
            },
            ApiRequest::Send(request) => {
                // Cleanup finished tasks
                self.cleanup_tasks();

                // Create the request
                match FetchRequest::post("http://localhost:30080/login/credentials").body(Cbor(&request)) {
                    Ok(bin) => self
                        .fetch_tasks
                        .push(self.fetch_service.fetch_binary(bin, self.link.send_back(Message))),
                    Err(_) => self.link.response(id, ApiResponse::RequestEncodeFailed),
                }
            }
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
