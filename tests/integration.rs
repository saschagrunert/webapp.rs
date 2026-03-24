#![cfg(feature = "ssr")]

use std::env;

use sqlx::postgres::PgPoolOptions;
use webapp::{auth, database};

async fn setup() {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect to test database");
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
    drop(pool);
    database::init().await.unwrap();
}

#[tokio::test]
async fn auth_and_session_flow() {
    setup().await;
    let pool = database::pool();

    // Register a user
    let hash = auth::hash_password("secret123").unwrap();
    database::create_user("testuser", &hash).await.unwrap();
    assert!(database::user_exists("testuser").await.unwrap());
    assert!(!database::user_exists("nobody").await.unwrap());

    // Verify password
    let stored_hash = database::get_password_hash("testuser")
        .await
        .unwrap()
        .unwrap();
    assert!(auth::verify_password("secret123", &stored_hash).unwrap());
    assert!(!auth::verify_password("wrong", &stored_hash).unwrap());
    assert!(
        database::get_password_hash("nobody")
            .await
            .unwrap()
            .is_none()
    );

    // Duplicate user fails
    assert!(database::create_user("testuser", &hash).await.is_err());

    // Create session
    let token = auth::create_token("testuser").unwrap();
    let expires = auth::token_expiry();
    database::create_session(&token, "testuser", expires)
        .await
        .unwrap();
    assert!(database::session_exists(&token).await.unwrap());

    // Verify token
    let username = auth::verify_token(&token).unwrap();
    assert_eq!(username, "testuser");

    // Invalid token fails
    assert!(auth::verify_token("garbage").is_err());

    // Password hashes use unique salts
    let h1 = auth::hash_password("same").unwrap();
    let h2 = auth::hash_password("same").unwrap();
    assert_ne!(h1, h2);

    // Renew session
    let new_token = auth::create_token("testuser").unwrap();
    let new_expires = auth::token_expiry();
    assert!(
        database::update_session(&token, &new_token, new_expires)
            .await
            .unwrap()
    );
    assert!(!database::session_exists(&token).await.unwrap());
    assert!(database::session_exists(&new_token).await.unwrap());

    // Logout
    assert!(database::delete_session(&new_token).await.unwrap());
    assert!(!database::session_exists(&new_token).await.unwrap());
    assert!(!database::delete_session(&new_token).await.unwrap());

    // Expired session cleanup
    let past = chrono::Utc::now() - chrono::Duration::hours(2);
    database::create_session("old_token", "testuser", past)
        .await
        .unwrap();
    let future = chrono::Utc::now() + chrono::Duration::hours(1);
    database::create_session("fresh_token", "testuser", future)
        .await
        .unwrap();

    let cleaned = database::delete_expired_sessions().await.unwrap();
    assert_eq!(cleaned, 1);
    assert!(!database::session_exists("old_token").await.unwrap());
    assert!(database::session_exists("fresh_token").await.unwrap());

    // Clean up for unit tests that may run after
    sqlx::query("DELETE FROM sessions")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM users")
        .execute(pool)
        .await
        .unwrap();
}
