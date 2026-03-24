use chrono::{DateTime, Utc};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::{env, sync::OnceLock};

static POOL: OnceLock<PgPool> = OnceLock::new();

pub async fn init() -> Result<(), sqlx::Error> {
    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://localhost/webapp".into());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::migrate!().run(&pool).await?;

    POOL.set(pool).expect("database pool already initialized");

    Ok(())
}

pub fn pool() -> &'static PgPool {
    POOL.get().expect("database pool not initialized")
}

// User management

pub async fn create_user(username: &str, password_hash: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO users (username, password_hash) VALUES ($1, $2)")
        .bind(username)
        .bind(password_hash)
        .execute(pool())
        .await?;
    Ok(())
}

pub async fn get_password_hash(username: &str) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT password_hash FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(pool())
            .await?;
    Ok(row.map(|r| r.0))
}

pub async fn user_exists(username: &str) -> Result<bool, sqlx::Error> {
    let row: Option<(i64,)> = sqlx::query_as("SELECT COUNT(*) FROM users WHERE username = $1")
        .bind(username)
        .fetch_optional(pool())
        .await?;
    Ok(row.is_some_and(|r| r.0 > 0))
}

// Session management

pub async fn create_session(
    token: &str,
    username: &str,
    expires_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO sessions (token, username, expires_at) VALUES ($1, $2, $3)")
        .bind(token)
        .bind(username)
        .bind(expires_at)
        .execute(pool())
        .await?;
    Ok(())
}

pub async fn session_exists(token: &str) -> Result<bool, sqlx::Error> {
    let row: Option<(i64,)> = sqlx::query_as("SELECT COUNT(*) FROM sessions WHERE token = $1")
        .bind(token)
        .fetch_optional(pool())
        .await?;
    Ok(row.is_some_and(|r| r.0 > 0))
}

pub async fn update_session(
    old_token: &str,
    new_token: &str,
    expires_at: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("UPDATE sessions SET token = $1, expires_at = $2 WHERE token = $3")
        .bind(new_token)
        .bind(expires_at)
        .bind(old_token)
        .execute(pool())
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn delete_session(token: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM sessions WHERE token = $1")
        .bind(token)
        .execute(pool())
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn delete_expired_sessions() -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM sessions WHERE expires_at < NOW()")
        .execute(pool())
        .await?;
    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("failed to connect to test database");
        // Drop existing tables and migration state to apply fresh schema
        sqlx::query("DROP TABLE IF EXISTS sessions")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DROP TABLE IF EXISTS users")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DROP TABLE IF EXISTS _sqlx_migrations")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("failed to run migrations");
        let _ = POOL.set(pool);
    }

    #[tokio::test]
    async fn database_operations() {
        setup().await;

        // User creation and lookup
        create_user("alice", "$argon2id$hash").await.unwrap();
        assert!(user_exists("alice").await.unwrap());
        assert!(!user_exists("bob").await.unwrap());

        // Password hash retrieval
        let hash = get_password_hash("alice").await.unwrap();
        assert_eq!(hash.as_deref(), Some("$argon2id$hash"));
        assert!(get_password_hash("nobody").await.unwrap().is_none());

        // Session lifecycle
        let expires = Utc::now() + chrono::Duration::hours(1);
        create_session("tok1", "alice", expires).await.unwrap();
        assert!(session_exists("tok1").await.unwrap());

        let updated = update_session("tok1", "tok2", expires).await.unwrap();
        assert!(updated);
        assert!(!session_exists("tok1").await.unwrap());
        assert!(session_exists("tok2").await.unwrap());

        let deleted = delete_session("tok2").await.unwrap();
        assert!(deleted);
        assert!(!session_exists("tok2").await.unwrap());

        // Deleting nonexistent session returns false
        assert!(!delete_session("nonexistent").await.unwrap());

        // Expired session cleanup
        let past = Utc::now() - chrono::Duration::hours(1);
        create_session("expired_tok", "alice", past).await.unwrap();
        let count = delete_expired_sessions().await.unwrap();
        assert!(count > 0);
        assert!(!session_exists("expired_tok").await.unwrap());
    }
}
