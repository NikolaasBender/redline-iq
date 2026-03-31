use axum::{
    routing::{get, post},
    Router, Json,
    http::HeaderMap,
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
        .route("/api/routes/active", post(set_active_route))
        .route("/api/upload", post(upload_gpx))
        .merge(auth::auth_router())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_segments(
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: HeaderMap,
) -> Json<serde_json::Value> {
    let token = headers.get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .unwrap_or("");

    let user_row: Option<String> = sqlx::query_scalar("SELECT active_route_id FROM users WHERE token = ?")
        .bind(token)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

    if let Some(route_id) = user_row {
        let route_row: Option<String> = sqlx::query_scalar("SELECT segments_json FROM routes WHERE id = ?")
            .bind(&route_id)
            .fetch_optional(&state.db)
            .await
            .unwrap_or(None);
        
        if let Some(segments_json) = route_row {
            let segments: serde_json::Value = serde_json::from_str(&segments_json).unwrap_or(serde_json::json!([]));
            return Json(serde_json::json!({
                "status": "success",
                "route_id": route_id,
                "segments": segments
            }));
        }
    }

    Json(serde_json::json!({
        "status": "error",
        "message": "Unauthorized or no active route selected"
    }))
}

#[derive(serde::Deserialize)]
pub struct SetActiveRoute {
    route_id: String,
}

async fn set_active_route(
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SetActiveRoute>,
) -> Json<serde_json::Value> {
    let token = headers.get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .unwrap_or("");
    
    let update = sqlx::query("UPDATE users SET active_route_id = ? WHERE token = ?")
        .bind(&payload.route_id)
        .bind(token)
        .execute(&state.db)
        .await;

    if update.is_ok() && update.unwrap().rows_affected() > 0 {
        Json(serde_json::json!({ "status": "success" }))
    } else {
        Json(serde_json::json!({ "status": "error", "message": "Unauthorized or user not found" }))
    }
}

async fn upload_gpx(
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: HeaderMap,
    body: String
) -> Json<serde_json::Value> {
    let token = headers.get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .unwrap_or("");

    let segments = gpx::parse_gpx_segments(&body);
    let segments_json = serde_json::to_string(&segments).unwrap_or_default();
    let route_id = uuid::Uuid::new_v4().to_string();

    let insert = sqlx::query("INSERT INTO routes (id, user_id, name, segments_json) VALUES (?, ?, ?, ?)")
        .bind(&route_id)
        .bind(token)
        .bind("Uploaded Route")
        .bind(&segments_json)
        .execute(&state.db)
        .await;

    if insert.is_ok() {
        Json(serde_json::json!({
            "status": "success",
            "route_id": route_id,
            "parsed_segments": segments
        }))
    } else {
        Json(serde_json::json!({ "status": "error", "message": "Failed to save route" }))
    }
}
