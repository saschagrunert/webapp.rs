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

pub async fn create_session(token: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO sessions (token) VALUES ($1)")
        .bind(token)
        .execute(pool())
        .await?;
    Ok(())
}

pub async fn update_session(old_token: &str, new_token: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE sessions SET token = $1 WHERE token = $2")
        .bind(new_token)
        .bind(old_token)
        .execute(pool())
        .await?;
    Ok(())
}

pub async fn delete_session(token: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sessions WHERE token = $1")
        .bind(token)
        .execute(pool())
        .await?;
    Ok(())
}
