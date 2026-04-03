use axum::{
    routing::{get, post},
    Router, Json,
    http::HeaderMap,
    middleware::{self, Next},
    extract::Request,
    response::Response,
};
use std::net::SocketAddr;
use tower_http::services::ServeDir;

mod gpx;
mod auth;
mod db;
mod ridewithgps;
mod course_export;

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
        .route("/api/routes", get(list_routes))
        .route("/api/routes/active", post(set_active_route))
        .route("/api/routes/import", post(import_rwgps_route))
        .route("/api/routes/:id/name", post(rename_route))
        .route("/api/routes/:id/viz", get(get_route_viz))
        .route("/api/routes/:id/course.gpx", get(get_course_gpx))
        .route("/api/upload", post(upload_gpx))
        .merge(auth::auth_router())
        .layer(middleware::from_fn(logging_middleware))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn logging_middleware(req: Request, next: Next) -> Response {
    println!("Request: {} {}", req.method(), req.uri());
    next.run(req).await
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
            let mut segments: serde_json::Value = serde_json::from_str(&segments_json).unwrap_or(serde_json::json!([]));
            
            // Optimization: Strip polylines to save Garmin DataField memory (-1001 error)
            if let Some(segs) = segments.as_array_mut() {
                for s in segs {
                    if let Some(obj) = s.as_object_mut() {
                        obj.remove("polyline");
                    }
                }
            }

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

    let (name, segments, polyline) = gpx::parse_gpx(&body);
    let segments_json = serde_json::to_string(&segments).unwrap_or_default();
    let polyline_json = serde_json::to_string(&polyline).unwrap_or_default();
    let route_id = uuid::Uuid::new_v4().to_string();

    let insert = sqlx::query("INSERT INTO routes (id, user_id, name, segments_json, full_polyline_json, source) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&route_id)
        .bind(token)
        .bind(&name)
        .bind(&segments_json)
        .bind(&polyline_json)
        .bind("gpx")
        .execute(&state.db)
        .await;

    if insert.is_ok() {
        Json(serde_json::json!({
            "status": "success",
            "route_id": route_id,
            "name": name,
            "parsed_segments": segments
        }))
    } else {
        Json(serde_json::json!({ "status": "error", "message": "Failed to save route" }))
    }
}
async fn list_routes(
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: HeaderMap,
) -> Json<serde_json::Value> {
    let token = headers.get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .unwrap_or("");

    let routes: Vec<serde_json::Value> = sqlx::query("SELECT id, name FROM routes WHERE user_id = ?")
        .bind(token)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r: sqlx::sqlite::SqliteRow| {
            use sqlx::Row;
            serde_json::json!({
                "id": r.get::<String, _>("id"),
                "name": r.get::<String, _>("name")
            })
        })
        .collect();

    Json(serde_json::json!({ "status": "success", "routes": routes }))
}

#[derive(serde::Deserialize)]
pub struct ImportRoute {
    url: String,
}

async fn import_rwgps_route(
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ImportRoute>,
) -> Json<serde_json::Value> {
    let token = headers.get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .unwrap_or("");

    let route_id = match ridewithgps::extract_id_from_url(&payload.url) {
        Some(id) => id,
        None => return Json(serde_json::json!({ "status": "error", "message": "Invalid RWGPS URL" })),
    };

    match ridewithgps::fetch_rwgps_route(&route_id).await {
        Ok(route) => {
            let segments_json = serde_json::to_string(&route.segments).unwrap_or_default();
            let polyline_json = serde_json::to_string(&route.full_polyline).unwrap_or_default();
            let new_uuid = uuid::Uuid::new_v4().to_string();

            let insert_result = sqlx::query("INSERT INTO routes (id, user_id, name, segments_json, full_polyline_json, rwgps_url, source) VALUES (?, ?, ?, ?, ?, ?, ?)")
                .bind(&new_uuid)
                .bind(token)
                .bind(&route.name)
                .bind(&segments_json)
                .bind(&polyline_json)
                .bind(&payload.url)
                .bind("rwgps")
                .execute(&state.db)
                .await;

            match insert_result {
                Ok(_) => {
                    println!("Route '{}' saved with id={}, user_id='{}', segments={}, polyline_points={}", route.name, new_uuid, token, route.segments.len(), route.full_polyline.len());
                    Json(serde_json::json!({ "status": "success", "route_id": new_uuid, "name": route.name }))
                }
                Err(e) => {
                    println!("ERROR saving route: {:?}", e);
                    Json(serde_json::json!({ "status": "error", "message": format!("Failed to save route: {}", e) }))
                }
            }
        }
        Err(e) => Json(serde_json::json!({ "status": "error", "message": e })),
    }
}

#[derive(serde::Deserialize)]
pub struct RenameRoute {
    name: String,
}

async fn rename_route(
    axum::extract::State(state): axum::extract::State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    headers: HeaderMap,
    Json(payload): Json<RenameRoute>,
) -> Json<serde_json::Value> {
    let token = headers.get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .unwrap_or("");

    let _ = sqlx::query("UPDATE routes SET name = ? WHERE id = ? AND user_id = ?")
        .bind(&payload.name)
        .bind(&id)
        .bind(token)
        .execute(&state.db)
        .await;

    Json(serde_json::json!({ "status": "success" }))
}

async fn get_route_viz(
    axum::extract::State(state): axum::extract::State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    let route = sqlx::query("SELECT name, segments_json, full_polyline_json FROM routes WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

    if let Some(r) = route {
        use sqlx::Row;
        let segments_json: String = r.get("segments_json");
        let polyline_json: String = r.get("full_polyline_json");
        let name: String = r.get("name");

        let segments: serde_json::Value = serde_json::from_str(&segments_json).unwrap_or(serde_json::json!([]));
        let polyline: serde_json::Value = serde_json::from_str(&polyline_json).unwrap_or(serde_json::json!([]));
        
        Json(serde_json::json!({
            "status": "success",
            "name": name,
            "segments": segments,
            "full_polyline": polyline
        }))
    } else {
        Json(serde_json::json!({ "status": "error", "message": "Route not found" }))
    }
}

async fn get_course_gpx(
    axum::extract::State(state): axum::extract::State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl axum::response::IntoResponse {
    let route = sqlx::query("SELECT name, segments_json, full_polyline_json FROM routes WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

    if let Some(r) = route {
        use sqlx::Row;
        let name: String = r.get("name");
        let segments_json: String = r.get("segments_json");
        let polyline_json: String = r.get("full_polyline_json");

        let segments: Vec<course_export::Segment> = serde_json::from_str(&segments_json).unwrap_or_default();
        let polyline: Vec<[f64; 2]> = serde_json::from_str(&polyline_json).unwrap_or_default();

        let gpx_content = course_export::generate_gpx_course(&name, &polyline, &segments);

        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::CONTENT_TYPE,
            axum::http::header::HeaderValue::from_static("application/gpx+xml"),
        );
        headers.insert(
            axum::http::header::CONTENT_DISPOSITION,
            axum::http::header::HeaderValue::from_str(&format!("attachment; filename=\"{}.gpx\"", name.replace(' ', "_"))).unwrap(),
        );

        (headers, gpx_content).into_response()
    } else {
        axum::http::StatusCode::NOT_FOUND.into_response()
    }
}
