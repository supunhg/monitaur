use std::sync::Arc;

use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use axum::{Json, Router, routing::get, routing::post};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct SetupRequest {
    password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    token: String,
    message: String,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub has_admin: bool,
    pub auth_enabled: bool,
}

pub fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/auth/setup", post(setup_handler))
        .route("/api/auth/login", post(login_handler))
        .route("/api/auth/status", get(status_handler))
}

async fn setup_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SetupRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<serde_json::Value>)> {
    if req.password.len() < 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Password must be at least 8 characters"})),
        ));
    }

    {
        let db = state.db.lock().await;
        if db.has_admin().unwrap_or(false) {
            return Err((
                StatusCode::CONFLICT,
                Json(serde_json::json!({"error": "Admin account already exists"})),
            ));
        }
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Hashing failed: {e}")})),
            )
        })?
        .to_string();

    let token = uuid::Uuid::new_v4().to_string();

    {
        let db = state.db.lock().await;
        db.set_password(&hash).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;
        db.create_token(&token).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;
    }

    info!("Admin account created");
    Ok(Json(AuthResponse {
        token,
        message: "Admin account created successfully".to_string(),
    }))
}

async fn login_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<serde_json::Value>)> {
    let stored_hash = {
        let db = state.db.lock().await;
        db.get_password_hash().ok_or((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "No admin account configured"})),
        ))?
    };

    let parsed_hash = PasswordHash::new(&stored_hash).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Corrupted password hash"})),
        )
    })?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Invalid password"})),
            )
        })?;

    let token = uuid::Uuid::new_v4().to_string();
    {
        let db = state.db.lock().await;
        db.create_token(&token).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;
    }

    info!("New login token issued");
    Ok(Json(AuthResponse {
        token,
        message: "Login successful".to_string(),
    }))
}

async fn status_handler(State(state): State<Arc<AppState>>) -> Json<StatusResponse> {
    let has_admin = {
        let db = state.db.lock().await;
        db.has_admin().unwrap_or(false)
    };

    Json(StatusResponse {
        has_admin,
        auth_enabled: state.auth_enabled,
    })
}

/// Auth middleware — checks Bearer token when auth is enabled.
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    if !state.auth_enabled {
        return Ok(next.run(req).await);
    }

    let auth = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    match auth {
        Some(token) => {
            let valid = state.db.lock().await.validate_token(&token).unwrap_or(false);
            if valid {
                Ok(next.run(req).await)
            } else {
                Err((
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({"error": "Invalid token"})),
                ))
            }
        }
        None => Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Missing Authorization header"})),
        )),
    }
}
