use axum::{
    extract::Query,
    routing::{get, post},
    Router, Json, response::Redirect
};
use serde::Deserialize;
use serde_json::Value;
use crate::AppState;
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl,
    AuthorizationCode, reqwest::async_http_client, TokenResponse,
};

fn oauth_client() -> BasicClient {
    // Falls back to blank strings if .env is missing, which safely fails instead of crashing
    let client_id = std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default();
    let client_secret = std::env::var("GOOGLE_CLIENT_SECRET").unwrap_or_default();
    let redirect_url = std::env::var("REDIRECT_URL")
        .unwrap_or_else(|_| "http://localhost:8080/auth/google/callback".to_string());

    BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string()).unwrap(),
        Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).unwrap())
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap())
}

pub fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/signup", post(signup))
        .route("/login", post(login))
        .route("/auth/google", get(google_login))
        .route("/auth/google/callback", get(google_callback))
}

async fn signup() -> Json<Value> {
    Json(serde_json::json!({ "status": "success", "message": "User signed up (placeholder)" }))
}

async fn login() -> Json<Value> {
    Json(serde_json::json!({ "status": "success", "message": "User logged in (placeholder)", "token": "mock_jwt_token" }))
}

async fn google_login() -> Redirect {
    let client = oauth_client();
    let (auth_url, _csrf_token) = client
        .authorize_url(oauth2::CsrfToken::new_random)
        .add_scope(oauth2::Scope::new("email".to_string()))
        .add_scope(oauth2::Scope::new("profile".to_string()))
        .url();

    // Redirect to the formulated Google OAuth url
    Redirect::temporary(auth_url.as_str())
}

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    code: String,
    state: String,
}

async fn google_callback(Query(query): Query<AuthRequest>) -> Json<Value> {
    let client = oauth_client();
    
    // Exchange the authorization code parameter for an access token
    match client.exchange_code(AuthorizationCode::new(query.code))
        .request_async(async_http_client).await {
        Ok(token) => {
            // Once successfully authenticated, return a synchronized Garmin token (using snippet for demo context)
            Json(serde_json::json!({ 
                "status": "success", 
                "message": "Google auth successful", 
                "google_token": token.access_token().secret(),
                "token": "SYNC-TOKEN-FROM-GOOGLE-123" 
            }))
        }
        Err(e) => {
            Json(serde_json::json!({ 
                "status": "error", 
                "message": format!("Token exchange failed: {:?}", e) 
            }))
        }
    }
}
