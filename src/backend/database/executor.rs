//! Everything related to database handling

use actix::prelude::*;
use backend::{
    database::{models, schema::sessions::dsl::*},
    server::ServerError,
};
use diesel::{
    delete, insert_into,
    prelude::*,
    r2d2::{ConnectionManager, Pool},
    update,
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
        // Retrieve the current connection
        let connection = &self.0.get()?;

        // Insert the session into the database
        debug!("Creating new session: {}", msg.id);
        let new_session = models::NewSession { id: &msg.id };
        insert_into(sessions).values(&new_session).execute(connection)?;

        // Return the inserted item
        sessions
            .filter(id.eq(&msg.id))
            .load(connection)?
            .pop()
            .ok_or_else(|| ServerError::InsertToken.into())
    }
}

/// The update session message
pub struct UpdateSession {
    pub old_id: String,
    pub new_id: String,
}

impl Message for UpdateSession {
    type Result = Result<models::Session, Error>;
}

impl Handler<UpdateSession> for DatabaseExecutor {
    type Result = Result<models::Session, Error>;

    fn handle(&mut self, msg: UpdateSession, _: &mut Self::Context) -> Self::Result {
        // Retrieve the current connection
        let connection = &self.0.get()?;

        // Update the session
        debug!("Updating session: {}", msg.old_id);
        update(sessions.filter(id.eq(&msg.old_id)))
            .set(id.eq(&msg.new_id))
            .execute(connection)?;

        // Return the updated item
        sessions
            .filter(id.eq(&msg.new_id))
            .load(connection)?
            .pop()
            .ok_or_else(|| ServerError::UpdateToken.into())
    }
}

/// The delete session message
pub struct DeleteSession {
    pub id: String,
}

impl Message for DeleteSession {
    type Result = Result<(), Error>;
}

impl Handler<DeleteSession> for DatabaseExecutor {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: DeleteSession, _: &mut Self::Context) -> Self::Result {
        // Retrieve the current connection
        let connection = &self.0.get()?;

        // Delete the session
        debug!("Deleting session: {}", msg.id);
        delete(sessions.filter(id.eq(&msg.id))).execute(connection)?;

        Ok(())
    }
}
