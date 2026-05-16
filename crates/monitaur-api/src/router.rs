use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::get;
use axum::{Json, Router};
use monitaur_core::metrics::SystemMetrics;
use monitaur_core::models::{InfraGraph, SecurityFinding, Service};
use monitaur_core::network::NetworkAnalysis;
use monitaur_core::visualization::TopologyGraph;
use serde::Serialize;
use tower_http::cors::CorsLayer;

use crate::auth;
use crate::state::AppState;

// ── Auth check helper ───────────────────────────────────────────

async fn check_auth(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    if !state.auth_enabled {
        return Ok(());
    }

    let auth = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    match auth {
        Some(token) => {
            let db = state.db.lock().await;
            if db.validate_token(&token).unwrap_or(false) {
                Ok(())
            } else {
                Err((StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Invalid token"}))))
            }
        }
        None => Err((StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Missing Authorization header"})))),
    }
}

// ── Router ─────────────────────────────────────────────────────

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .merge(auth::auth_routes())
        .route("/api/scan", get(run_scan))
        .route("/api/services", get(list_services))
        .route("/api/services/{id}", get(get_service))
        .route("/api/metrics", get(get_metrics))
        .route("/api/metrics/history", get(get_metrics_history))
        .route("/api/security", get(get_security))
        .route("/api/security/findings", get(list_findings))
        .route("/api/network", get(get_network))
        .route("/api/visualization", get(get_visualization))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

// ── Handlers ───────────────────────────────────────────────────

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[derive(Serialize)]
struct ScanResponse {
    discovery: InfraGraph,
    security: Vec<SecurityFinding>,
    network: NetworkAnalysis,
    visualization: TopologyGraph,
}

async fn run_scan(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<ScanResponse>, (StatusCode, Json<serde_json::Value>)> {
    check_auth(&state, &headers).await?;

    // Force fresh discovery for scan (ignore cache)
    let graph = state.force_discover().await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Discovery failed"}))))?;

    let security = state.security();
    let findings = security.analyze(&graph.services).await;
    {
        let db = state.db.lock().await;
        for finding in &findings {
            db.save_finding(finding).map_err(|_e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Internal server error"}))))?;
        }
    }
    {
        let mut meta = state.metadata.lock().await;
        meta.snapshot_infra();
    }

    let network = state.network();
    let network_analysis = network.analyze().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Network analysis failed"}))))?;
    {
        let db = state.db.lock().await;
        db.save_network_analysis(&network_analysis).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Database error"}))))?;
    }

    let viz = state.visualization();
    let topology = viz.render(&graph);

    Ok(Json(ScanResponse {
        discovery: graph,
        security: findings,
        network: network_analysis,
        visualization: topology,
    }))
}

async fn list_services(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<Service>>, (StatusCode, Json<serde_json::Value>)> {
    check_auth(&state, &headers).await?;

    let graph = state.discover().await.map_err(|_e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Internal server error"}))))?;
    Ok(Json(graph.services))
}

async fn get_service(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Service>, (StatusCode, Json<serde_json::Value>)> {
    check_auth(&state, &headers).await?;

    let _graph = state.discover().await.map_err(|_e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Internal server error"}))))?;
    let meta = state.metadata.lock().await;
    meta.index
        .by_id(&id)
        .map(|s| (*s).clone())
        .map(Json)
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": format!("Service {id} not found")}))))
}

#[derive(Serialize)]
struct MetricsResponse {
    system: Option<SystemMetrics>,
    containers: Vec<monitaur_core::metrics::ContainerMetrics>,
}

async fn get_metrics(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<MetricsResponse>, (StatusCode, Json<serde_json::Value>)> {
    check_auth(&state, &headers).await?;

    let graph = state.discover().await.map_err(|_e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Internal server error"}))))?;

    let mut monitoring = state.monitoring.lock().await;
    let snapshot = monitoring.snapshot(&graph.services).await.map_err(|_e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Internal server error"}))))?;

    {
        let db = state.db.lock().await;
        db.save_metrics_snapshot(&snapshot).map_err(|_e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Internal server error"}))))?;
    }
    {
        let mut meta = state.metadata.lock().await;
        meta.snapshot_metrics(snapshot.clone());
    }

    Ok(Json(MetricsResponse {
        system: snapshot.system,
        containers: snapshot.containers,
    }))
}

async fn get_security(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<SecurityFinding>>, (StatusCode, Json<serde_json::Value>)> {
    check_auth(&state, &headers).await?;

    let graph = state.discover().await.map_err(|_e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Internal server error"}))))?;

    let security = state.security();
    let findings = security.analyze(&graph.services).await;

    {
        let db = state.db.lock().await;
        for finding in &findings {
            db.save_finding(finding).map_err(|_e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Internal server error"}))))?;
        }
    }

    Ok(Json(findings))
}

async fn get_network(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<NetworkAnalysis>, (StatusCode, Json<serde_json::Value>)> {
    check_auth(&state, &headers).await?;

    let network = state.network();
    let analysis = network.analyze().map_err(|_e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Internal server error"}))))?;
    {
        let db = state.db.lock().await;
        db.save_network_analysis(&analysis).map_err(|_e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Internal server error"}))))?;
    }
    Ok(Json(analysis))
}

async fn get_visualization(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<TopologyGraph>, (StatusCode, Json<serde_json::Value>)> {
    check_auth(&state, &headers).await?;

    let graph = state.discover().await.map_err(|_e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Internal server error"}))))?;
    let viz = state.visualization();
    let topology = viz.render(&graph);
    Ok(Json(topology))
}

async fn get_metrics_history(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<monitaur_core::metrics::MetricsSnapshot>>, (StatusCode, Json<serde_json::Value>)> {
    check_auth(&state, &headers).await?;

    let db = state.db.lock().await;
    let history = db.list_metrics_history(60).map_err(|_e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Internal server error"})))
    })?;
    Ok(Json(history))
}

async fn list_findings(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<SecurityFinding>>, (StatusCode, Json<serde_json::Value>)> {
    check_auth(&state, &headers).await?;

    let db = state.db.lock().await;
    let findings = db.list_findings(100, None).map_err(|_e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Internal server error"})))
    })?;
    Ok(Json(findings))
}
