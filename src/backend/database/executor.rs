//! Everything related to database handling

use actix::prelude::*;
use actix_web::error::{Error, ErrorInternalServerError};
use backend::database::{models, schema};
use diesel::{
    self,
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};

/// The database executor actor
pub struct DatabaseExecutor(pub Pool<ConnectionManager<SqliteConnection>>);

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
        use self::schema::sessions::dsl::*;

        let new_session = models::NewSession { id: &msg.id };

        let connection: &SqliteConnection = &self.0.get().unwrap();

        diesel::insert_into(sessions)
            .values(&new_session)
            .execute(connection)
            .map_err(|_| ErrorInternalServerError("Error inserting session"))?;

        let mut items = sessions
            .filter(id.eq(&msg.id))
            .load::<models::Session>(connection)
            .map_err(|_| ErrorInternalServerError("Error loading session"))?;

        Ok(items.pop().unwrap())
    }
}
