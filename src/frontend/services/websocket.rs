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
use API_URL;

lazy_static! {
    /// Create a single websocket connection per application instance
    static ref WEBSOCKET: WebSocket = WebSocket::new(API_URL).expect("Unable to connect to websocket");
}

/// A status of a websocket connection. Used for status notification.
pub enum WebSocketStatus {
    /// Used when a websocket connection was opened
    Opened,

    /// Used when a websocket connection was closed
    Closed,

    /// Used when a websocket connection was failed
    Error,
}

/// A handle to control current websocket connection. Implements `Task` and could be canceled.
pub struct WebSocketService {
    notification: Callback<WebSocketStatus>,
}

impl WebSocketService {
    /// Connects to a server by a websocket connection. Needs two functions to generate data and
    /// notification messages.
    pub fn new(callback: Callback<Vec<u8>>, notification: Callback<WebSocketStatus>) -> Self {
        // Set the websocket to binary mode
        WEBSOCKET.set_binary_type(SocketBinaryType::ArrayBuffer);

        // Create notification callbacks
        let n = notification.clone();
        WEBSOCKET.add_event_listener(move |_: SocketOpenEvent| {
            n.emit(WebSocketStatus::Opened);
        });
        let n = notification.clone();
        WEBSOCKET.add_event_listener(move |_: SocketCloseEvent| {
            n.emit(WebSocketStatus::Closed);
        });
        let n = notification.clone();
        WEBSOCKET.add_event_listener(move |_: SocketErrorEvent| {
            n.emit(WebSocketStatus::Error);
        });

        // Add data callback
        WEBSOCKET.add_event_listener(move |event: SocketMessageEvent| {
            if let Some(bytes) = event.data().into_array_buffer() {
                callback.emit(bytes.into());
            }
        });

        Self { notification }
    }

    /// Sends binary data to a websocket connection.
    pub fn send(&mut self, data: &[u8]) {
        if WEBSOCKET.send_bytes(data).is_err() {
            self.notification.emit(WebSocketStatus::Error);
        }
    }
}

impl Task for WebSocketService {
    /// Test wheter the websocket connection is active
    fn is_active(&self) -> bool {
        WEBSOCKET.ready_state() == SocketReadyState::Open
    }

    // Close the websocket connection
    fn cancel(&mut self) {
        WEBSOCKET.close();
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
