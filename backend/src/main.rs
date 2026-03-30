use axum::{
    routing::{get, post},
    Router, Json,
};
use std::net::SocketAddr;
use tower_http::services::ServeDir;

mod gpx;
mod auth;
mod db;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::sqlite::SqlitePool,
}

#[tokio::main]
async fn main() {
    let pool = db::init_db().await;
    let state = AppState { db: pool };

    let app = Router::new()
        .nest_service("/", ServeDir::new("public"))
        .route("/api/segments", get(get_segments))
        .route("/api/upload", post(upload_gpx))
        .merge(auth::auth_router())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_segments() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "success",
        "segments": []
    }))
}

async fn upload_gpx(body: String) -> Json<serde_json::Value> {
    let segments = gpx::parse_gpx_segments(&body);
    Json(serde_json::json!({
        "status": "success",
        "parsed_segments": segments
    }))
}
