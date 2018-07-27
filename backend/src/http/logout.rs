//! The session based login request

use actix::{dev::ToEnvelope, prelude::*};
use actix_web::{AsyncResponder, HttpRequest, HttpResponse};
use cbor::{CborRequest, CborResponseBuilder};
use database::DeleteSession;
use futures::Future;
use http::FutureResponse;
use server::State;
use webapp::protocol::{model::Session, request, response};

pub fn logout<T: Actor>(http_request: &HttpRequest<State<T>>) -> FutureResponse
where
    T: Actor + Handler<DeleteSession>,
    <T as Actor>::Context: ToEnvelope<T, DeleteSession>,
{
    let request_clone = http_request.clone();
    CborRequest::new(http_request)
        .from_err()
        // Remove the session from the database
        .and_then(move |request::Logout(Session{token})| {
            debug!("Session token {} wants to be logged out", token);
            request_clone
                .state()
                .database
                .send(DeleteSession(token))
                .from_err()
                .and_then(|result| match result {
                    Ok(()) => Ok(HttpResponse::Ok().cbor(response::Logout)?),
                    Err(_) => Ok(HttpResponse::InternalServerError().into()),
                })
        })
        .responder()
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestServer;
    use database::DatabaseError;
    use http::tests::{execute_request, state, DatabaseExecutorMock};
    use serde_cbor::to_vec;
    use webapp::protocol::{model::Session, request};

    impl Handler<DeleteSession> for DatabaseExecutorMock {
        type Result = Result<(), DatabaseError>;
        fn handle(&mut self, _: DeleteSession, _: &mut Self::Context) -> Self::Result {
            Ok(())
        }
    }

    fn create_testserver() -> TestServer {
        TestServer::build_with_state(state).start(|app| app.handler(logout))
    }

    #[test]
    fn succeed_to_logout() {
        // Given
        let mut server = create_testserver();
        let body = to_vec(&request::Logout(Session {
            token: "any-token".to_owned(),
        })).unwrap();

        // When
        let response = execute_request(&mut server, body);

        // Then
        assert!(response.status().is_success());
    }
}
