//! Basic models

#[cfg(feature = "backend")]
use schema::sessions;

#[cfg_attr(feature = "backend", derive(Insertable, Queryable))]
#[cfg_attr(feature = "backend", table_name = "sessions")]
#[derive(Deserialize, Serialize)]
/// A session representation
pub struct Session {
    /// The actual session token
    pub token: String,
}
