//! Database models based on schema definitions

use backend::database::schema::sessions;

#[derive(Serialize, Queryable)]
pub struct Session {
    pub id: String,
}

#[derive(Insertable)]
#[table_name = "sessions"]
pub struct NewSession<'a> {
    pub id: &'a str,
}
