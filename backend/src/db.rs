use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

pub async fn init_db() -> SqlitePool {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_string());
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to create SQLite connection pool");

    sqlx::query("
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            token TEXT UNIQUE,
            active_route_id TEXT
        );
        CREATE TABLE IF NOT EXISTS routes (
            id TEXT PRIMARY KEY,
            user_id TEXT,
            name TEXT,
            segments_json TEXT
        );
    ")
    .execute(&pool)
    .await
    .expect("Failed to initialize database schema");

    pool
}
