//! Everything related to database handling

use actix::prelude::*;
use backend::{
    database::{models, schema::sessions::dsl::*},
    server::ServerError,
};
use diesel::{
    insert_into,
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};
use failure::Error;

/// The database executor actor
pub struct DatabaseExecutor(pub Pool<ConnectionManager<PgConnection>>);

impl Actor for DatabaseExecutor {
    type Context = SyncContext<Self>;
}

/// The create session message
pub struct CreateSession {
    pub id: String,
}

impl Message for CreateSession {
    type Result = Result<models::Session, Error>;
}

impl Handler<CreateSession> for DatabaseExecutor {
    type Result = Result<models::Session, Error>;

    fn handle(&mut self, msg: CreateSession, _: &mut Self::Context) -> Self::Result {
        let new_session = models::NewSession { id: &msg.id };
        let connection = &self.0.get()?;
        insert_into(sessions).values(&new_session).execute(connection)?;
        let mut items = sessions.filter(id.eq(&msg.id)).load(connection)?;
        items.pop().ok_or(ServerError::Internal.into())
    }
}
