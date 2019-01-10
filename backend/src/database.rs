//! Everything related to database handling

use actix::prelude::*;
use diesel::{
    delete, insert_into,
    prelude::*,
    r2d2::{ConnectionManager, Pool},
    update,
};
use failure::Fallible;
use log::debug;
use webapp::{protocol::model::Session, schema::sessions::dsl::*};

/// The database executor actor
pub struct DatabaseExecutor(pub Pool<ConnectionManager<PgConnection>>);

impl Actor for DatabaseExecutor {
    type Context = SyncContext<Self>;
}

/// The create session message
pub struct CreateSession(pub String);

impl Message for CreateSession {
    type Result = Fallible<Session>;
}

impl Handler<CreateSession> for DatabaseExecutor {
    type Result = Fallible<Session>;

    fn handle(&mut self, msg: CreateSession, _: &mut Self::Context) -> Self::Result {
        // Insert the session into the database
        debug!("Creating new session: {}", msg.0);
        Ok(insert_into(sessions)
            .values(&Session::new(msg.0))
            .get_result::<Session>(&self.0.get()?)?)
    }
}

/// The update session message
pub struct UpdateSession {
    /// The old session token
    pub old_token: String,

    /// The new session token
    pub new_token: String,
}

impl Message for UpdateSession {
    type Result = Fallible<Session>;
}

impl Handler<UpdateSession> for DatabaseExecutor {
    type Result = Fallible<Session>;

    fn handle(&mut self, msg: UpdateSession, _: &mut Self::Context) -> Self::Result {
        // Update the session
        debug!("Updating session: {}", msg.old_token);
        Ok(update(sessions.filter(token.eq(&msg.old_token)))
            .set(token.eq(&msg.new_token))
            .get_result::<Session>(&self.0.get()?)?)
    }
}

/// The delete session message, needs a token
pub struct DeleteSession(pub String);

impl Message for DeleteSession {
    type Result = Fallible<()>;
}

impl Handler<DeleteSession> for DatabaseExecutor {
    type Result = Fallible<()>;

    fn handle(&mut self, msg: DeleteSession, _: &mut Self::Context) -> Self::Result {
        // Delete the session
        debug!("Deleting session: {}", msg.0);
        delete(sessions.filter(token.eq(&msg.0))).execute(&self.0.get()?)?;
        Ok(())
    }
}
