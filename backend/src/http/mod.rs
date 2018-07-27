//! HTTP message handling

pub mod login_credentials;
pub mod login_session;
pub mod logout;

pub use http::login_credentials::login_credentials;
pub use http::login_session::login_session;
pub use http::logout::logout;

use actix_web::{error::Error, HttpResponse};
use futures::Future;

pub type FutureResponse = Box<Future<Item = HttpResponse, Error = Error>>;

#[cfg(test)]
mod tests {
    use actix::prelude::*;
    use actix_web::{client::ClientResponse, test::TestServer};
    use server::State;

    /// The mock database executor actor
    pub struct DatabaseExecutorMock;

    impl Actor for DatabaseExecutorMock {
        type Context = SyncContext<Self>;
    }

    pub fn state() -> State<DatabaseExecutorMock> {
        State {
            database: SyncArbiter::start(1, move || DatabaseExecutorMock),
        }
    }

    pub fn execute_request(server: &mut TestServer, body: Vec<u8>) -> ClientResponse {
        let request = server.post().body(body).unwrap();
        server.execute(request.send()).unwrap()
    }
}
