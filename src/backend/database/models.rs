//! Database models based on schema definitions

use backend::database::schema::sessions;

#[derive(Insertable, Serialize, Queryable)]
#[table_name = "sessions"]
pub struct Session {
    pub id: String,
}
