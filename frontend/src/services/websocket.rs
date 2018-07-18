//! A custom websocket service
//! [`WebSocket` Protocol](https://tools.ietf.org/html/rfc6455).

use std::collections::HashSet;
use stdweb::{
    traits::IMessageEvent,
    web::{
        event::{SocketCloseEvent, SocketErrorEvent, SocketMessageEvent, SocketOpenEvent},
        IEventTarget, SocketBinaryType, WebSocket,
    },
};
use yew::prelude::worker::*;

#[derive(Clone, Deserialize, Serialize)]
/// Available WebSocket responses
pub enum WebSocketResponse {
    Closed,
    Error,
    Opened,
    Data(Vec<u8>),
}

impl Transferable for WebSocketResponse {}

#[derive(Deserialize, Serialize)]
/// The WebSocket data request
pub struct WebSocketRequest(pub Vec<u8>);

impl Transferable for WebSocketRequest {}

/// The WebSocketAgent which handles a websocket connection per tab
pub struct WebSocketAgent {
    link: AgentLink<WebSocketAgent>,
    subscribers: HashSet<HandlerId>,
    websocket: WebSocket,
}

impl Agent for WebSocketAgent {
    type Reach = Context;
    type Message = WebSocketResponse;
    type Input = WebSocketRequest;
    type Output = WebSocketResponse;

    /// Creates a new WebSocketAgent
    fn create(link: AgentLink<Self>) -> Self {
        // Create the WebSocket connection
        let websocket = WebSocket::new(env!("WS_URL")).expect("Unable to connect to websocket");

        // Set the websocket to binary mode
        websocket.set_binary_type(SocketBinaryType::ArrayBuffer);

        // Create notification callbacks
        let notification_callback = link.send_back(|data| data);
        let n = notification_callback.clone();
        websocket.add_event_listener(move |_: SocketOpenEvent| {
            n.emit(WebSocketResponse::Opened);
        });
        let n = notification_callback.clone();
        websocket.add_event_listener(move |_: SocketCloseEvent| {
            n.emit(WebSocketResponse::Closed);
        });
        let n = notification_callback.clone();
        websocket.add_event_listener(move |_: SocketErrorEvent| {
            n.emit(WebSocketResponse::Error);
        });

        // Add data callback
        let data_callback = link.send_back(|data| WebSocketResponse::Data(data));
        websocket.add_event_listener(move |event: SocketMessageEvent| {
            if let Some(bytes) = event.data().into_array_buffer() {
                data_callback.emit(bytes.into());
            }
        });

        // Return the instance
        Self {
            link,
            subscribers: HashSet::new(),
            websocket,
        }
    }

    /// Internal update mechanism based on messages
    fn update(&mut self, msg: Self::Message) {
        // Inform all subscribers
        for who in self.subscribers.iter() {
            self.link.response(*who, msg.clone());
        }
    }

    /// Handle incoming data requests
    fn handle(&mut self, msg: Self::Input, _: HandlerId) {
        let WebSocketRequest(data) = msg;
        if self.websocket.send_bytes(&data).is_err() {
            self.update(WebSocketResponse::Error);
        }
    }

    /// Add a new client to the pool of connections to this agent
    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    /// Remove a client from the pool of connections of this agent
    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}
