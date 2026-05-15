use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use monitaur_core::error::EngineError;
use monitaur_core::metrics::SystemMetrics;
use monitaur_core::models::{InfraGraph, SecurityFinding, Service};
use monitaur_core::network::NetworkAnalysis;
use monitaur_core::visualization::TopologyGraph;
use serde::Serialize;
use tower_http::cors::CorsLayer;

use crate::app_state::AppState;

// ── Error wrapper ──────────────────────────────────────────────

struct ApiError(EngineError);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self.0 {
            EngineError::Discovery(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            EngineError::Monitoring(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            EngineError::Security(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            EngineError::Network(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            EngineError::Visualization(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            EngineError::Metadata(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            EngineError::Persistence(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            EngineError::Io(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };
        (status, Json(serde_json::json!({"error": message}))).into_response()
    }
}

type ApiResult<T> = Result<Json<T>, ApiError>;

// ── Router ─────────────────────────────────────────────────────

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/scan", get(run_scan))
        .route("/api/services", get(list_services))
        .route("/api/services/{id}", get(get_service))
        .route("/api/metrics", get(get_metrics))
        .route("/api/security", get(get_security))
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

async fn run_scan(State(state): State<Arc<AppState>>) -> ApiResult<ScanResponse> {
    let discovery = state.discovery();
    let graph = discovery.discover().await.map_err(ApiError)?;

    {
        let mut meta = state.metadata.lock().await;
        meta.update(graph.clone());
    }

    {
        let db = state.db.lock().await;
        db.save_infra_graph(&graph).map_err(ApiError)?;
    }

    let security = state.security();
    let findings = security.analyze(&graph.services).await;
    {
        let db = state.db.lock().await;
        for finding in &findings {
            db.save_finding(finding).map_err(ApiError)?;
        }
    }
    {
        let mut meta = state.metadata.lock().await;
        meta.snapshot_infra();
    }

    let network = state.network();
    let network_analysis = network.analyze().map_err(ApiError)?;
    {
        let db = state.db.lock().await;
        db.save_network_analysis(&network_analysis)
            .map_err(ApiError)?;
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

async fn list_services(State(state): State<Arc<AppState>>) -> ApiResult<Vec<Service>> {
    let discovery = state.discovery();
    let graph = discovery.discover().await.map_err(ApiError)?;

    {
        let mut meta = state.metadata.lock().await;
        meta.update(graph.clone());
    }

    Ok(Json(graph.services))
}

async fn get_service(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Service> {
    let discovery = state.discovery();
    let _graph = discovery.discover().await.map_err(ApiError)?;

    let meta = state.metadata.lock().await;
    meta.index
        .by_id(&id)
        .cloned()
        .map(Json)
        .ok_or_else(|| ApiError(EngineError::Discovery(format!("Service {id} not found"))))
}

#[derive(Serialize)]
struct MetricsResponse {
    system: Option<SystemMetrics>,
    containers: Vec<monitaur_core::metrics::ContainerMetrics>,
}

async fn get_metrics(State(state): State<Arc<AppState>>) -> ApiResult<MetricsResponse> {
    let discovery = state.discovery();
    let graph = discovery.discover().await.map_err(ApiError)?;

    let mut monitoring = state.monitoring.lock().await;
    let snapshot = monitoring
        .snapshot(&graph.services)
        .await
        .map_err(ApiError)?;

    {
        let db = state.db.lock().await;
        db.save_metrics_snapshot(&snapshot).map_err(ApiError)?;
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

async fn get_security(State(state): State<Arc<AppState>>) -> ApiResult<Vec<SecurityFinding>> {
    let discovery = state.discovery();
    let graph = discovery.discover().await.map_err(ApiError)?;

    let security = state.security();
    let findings = security.analyze(&graph.services).await;

    {
        let db = state.db.lock().await;
        for finding in &findings {
            db.save_finding(finding).map_err(ApiError)?;
        }
    }

    Ok(Json(findings))
}

async fn get_network(State(state): State<Arc<AppState>>) -> ApiResult<NetworkAnalysis> {
    let network = state.network();
    let analysis = network.analyze().map_err(ApiError)?;

    {
        let db = state.db.lock().await;
        db.save_network_analysis(&analysis).map_err(ApiError)?;
    }

    Ok(Json(analysis))
}

async fn get_visualization(State(state): State<Arc<AppState>>) -> ApiResult<TopologyGraph> {
    let discovery = state.discovery();
    let graph = discovery.discover().await.map_err(ApiError)?;

    let viz = state.visualization();
    let topology = viz.render(&graph);

    Ok(Json(topology))
}
