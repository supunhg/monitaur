use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};

use monitaur_api::{AppState, create_router};
use tauri::Emitter;
use tracing::error;

pub struct DesktopState {
    pub api_port: AtomicU16,
    pub app_state: Arc<AppState>,
}

impl DesktopState {
    fn new(db_path: &str) -> Result<Self, monitaur_core::error::EngineError> {
        let app_state = AppState::new(db_path, false)?;
        Ok(Self {
            api_port: AtomicU16::new(0),
            app_state,
        })
    }
}

#[tauri::command]
fn get_api_port(state: tauri::State<'_, DesktopState>) -> u16 {
    state.api_port.load(Ordering::Relaxed)
}

#[tauri::command]
fn health() -> serde_json::Value {
    serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    })
}

async fn start_api(state: &DesktopState) -> Result<u16, Box<dyn std::error::Error>> {
    let app = create_router(state.app_state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    state.api_port.store(port, Ordering::Relaxed);

    tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });

    Ok(port)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = match DesktopState::new("monitaur.db") {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to initialize desktop app: {e}");
            return;
        }
    };
    let state = Arc::new(state);

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(state.clone())
        .setup(move |app| {
            let handle = app.handle().clone();
            let state_clone = state.clone();
            tauri::async_runtime::spawn(async move {
                match start_api(state_clone.as_ref()).await {
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
        .build(tauri::generate_context!());

    match app {
        Ok(app) => {
            app.run(|_handle, _event| {});
        }
        Err(e) => {
            error!("Failed to build Tauri application: {e}");
        }
    }
}
