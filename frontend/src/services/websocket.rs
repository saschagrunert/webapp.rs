//! A custom websocket service
//! [`WebSocket` Protocol](https://tools.ietf.org/html/rfc6455).

use stdweb::{
    traits::IMessageEvent,
    web::{
        event::{SocketCloseEvent, SocketErrorEvent, SocketMessageEvent, SocketOpenEvent},
        IEventTarget, SocketBinaryType, SocketReadyState, WebSocket,
    },
};
use yew::{callback::Callback, services::Task};

/// Available WebSocket responses
pub enum WebSocketResponse {
    Close,
    Error,
    Open,
    Data(Vec<u8>),
}

/// A handle to control current websocket connection. Implements `Task` and could be canceled.
pub struct WebSocketService {
    websocket: WebSocket,
    callbacks: Callback<WebSocketResponse>,
}

impl WebSocketService {
    /// Connects to a server by a websocket connection. Needs two functions to generate data and
    /// notification messages.
    pub fn new(callbacks: Callback<WebSocketResponse>) -> Self {
        // Crate the WebSocket connection
        let websocket = WebSocket::new(env!("WS_URL")).expect("Unable to connect to websocket");

        // Set the websocket to binary mode
        websocket.set_binary_type(SocketBinaryType::ArrayBuffer);

        // Create notification callbacks
        let n = callbacks.clone();
        websocket.add_event_listener(move |_: SocketOpenEvent| {
            n.emit(WebSocketResponse::Open);
        });
        let n = callbacks.clone();
        websocket.add_event_listener(move |_: SocketCloseEvent| {
            n.emit(WebSocketResponse::Close);
        });
        let n = callbacks.clone();
        websocket.add_event_listener(move |_: SocketErrorEvent| {
            n.emit(WebSocketResponse::Error);
        });

        // Add data callback
        let n = callbacks.clone();
        websocket.add_event_listener(move |event: SocketMessageEvent| {
            if let Some(bytes) = event.data().into_array_buffer() {
                n.emit(WebSocketResponse::Data(bytes.into()));
            }
        });

        Self { callbacks, websocket }
    }

    /// Sends binary data to a websocket connection.
    pub fn send(&mut self, data: &[u8]) {
        if self.websocket.send_bytes(data).is_err() {
            self.callbacks.emit(WebSocketResponse::Error);
        }
    }
}

impl Task for WebSocketService {
    /// Test wheter the websocket connection is active
    fn is_active(&self) -> bool {
        self.websocket.ready_state() == SocketReadyState::Open
    }

    // Close the websocket connection
    fn cancel(&mut self) {
        self.websocket.close();
    }
}

impl Drop for WebSocketService {
    /// Close this connection on drop
    fn drop(&mut self) {
        if self.is_active() {
            self.cancel();
        }
    }
}
