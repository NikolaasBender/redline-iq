use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

pub async fn init_db() -> SqlitePool {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_string());
    
    if db_url.starts_with("sqlite:") && !db_url.contains(":memory:") {
        let path = db_url.trim_start_matches("sqlite:").trim_start_matches("//");
        println!("Initializing database at: {}", path);
        if let Some(parent) = std::path::Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).expect("Failed to create database directory");
            }
        }
    }

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
            segments_json TEXT,
            full_polyline_json TEXT,
            rwgps_url TEXT,
            source TEXT
        );
    ")
    .execute(&pool)
    .await
    .expect("Failed to initialize database schema");

    // Migrations: add columns that may be missing from older schemas
    let migrations = vec![
        "ALTER TABLE routes ADD COLUMN full_polyline_json TEXT",
        "ALTER TABLE routes ADD COLUMN rwgps_url TEXT",
        "ALTER TABLE routes ADD COLUMN source TEXT",
    ];
    for sql in migrations {
        let _ = sqlx::query(sql).execute(&pool).await; // ignore "duplicate column" errors
    }

    pool
}
