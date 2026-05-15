use std::sync::Arc;

use axum::Router;
use monitaur_persistence::PersistenceEngine;
use monitaur_discovery::DiscoveryEngine;
use monitaur_metadata::MetadataEngine;
use monitaur_monitoring::MonitoringEngine;
use monitaur_network::NetworkIntelligenceEngine;
use monitaur_security::SecurityEngine;
use monitaur_visualization::VisualizationEngine;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

/// Shared application state for Tauri commands.
pub struct DesktopState {
    pub api_port: u16,
    pub db: Mutex<PersistenceEngine>,
    pub monitoring: Mutex<MonitoringEngine>,
    pub metadata: Mutex<MetadataEngine>,
}

impl DesktopState {
    fn new(db_path: &str) -> Result<Self, monitaur_core::error::EngineError> {
        let db = PersistenceEngine::open(db_path)?;
        Ok(Self {
            api_port: 0,
            db: Mutex::new(db),
            monitoring: Mutex::new(MonitoringEngine::new().with_poll_interval(5)),
            metadata: Mutex::new(MetadataEngine::new()),
        })
    }
}

/// Tauri command: get the API server port.
#[tauri::command]
fn get_api_port(state: tauri::State<'_, DesktopState>) -> u16 {
    state.api_port
}

/// Health check command.
#[tauri::command]
fn health() -> serde_json::Value {
    serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    })
}

/// Start the embedded axum API server on a random port.
async fn start_api(state: Arc<DesktopState>) -> Result<u16, Box<dyn std::error::Error>> {
    let app: Router<()> = Router::new()
        .route("/api/health", axum::routing::get(|| async {
            axum::Json(serde_json::json!({"status": "ok", "version": "0.1.0"}))
        }))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    state.api_port = port;

    tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });

    Ok(port)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = DesktopState::new("monitaur.db").expect("Failed to open database");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = handle.state::<DesktopState>();
                match start_api(state.inner()).await {
                    Ok(port) => {
                        tracing::info!("API server started on port {port}");
                        let _ = handle.emit("api-ready", port);
                    }
                    Err(e) => {
                        tracing::error!("Failed to start API server: {e}");
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![health, get_api_port])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
