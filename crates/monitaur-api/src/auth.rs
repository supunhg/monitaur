use std::sync::Arc;

use argon2::Argon2;
use argon2::password_hash::{
    PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng,
};
use axum::extract::State;
use axum::http::StatusCode;
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
    if req.password.len() > 128 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Password must not exceed 128 characters"})),
        ));
    }

    {
        let db = state.db.lock().await;
        if db.has_admin().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Database error"})),
            )
        })? {
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
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Password hashing failed"})),
            )
        })?
        .to_string();

    let token = uuid::Uuid::new_v4().to_string();

    {
        let db = state.db.lock().await;
        db.set_password(&hash).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Database error"})),
            )
        })?;
        db.create_token(&token).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Database error"})),
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
        db.create_token(&token).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Database error"})),
            )
        })?;
    }

    info!("New login token issued");
    Ok(Json(AuthResponse {
        token,
        message: "Login successful".to_string(),
    }))
}

async fn status_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<StatusResponse>, (StatusCode, Json<serde_json::Value>)> {
    let has_admin = {
        let db = state.db.lock().await;
        db.has_admin().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Database error"})),
            )
        })?
    };

    Ok(Json(StatusResponse {
        has_admin,
        auth_enabled: state.auth_enabled,
    }))
}
