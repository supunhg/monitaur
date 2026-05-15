mod api;
mod app_state;
mod auth;

use std::net::SocketAddr;

use clap::{Parser, Subcommand};
use monitaur_core::error::EngineResult;
use monitaur_discovery::DiscoveryEngine;
use monitaur_metadata::MetadataEngine;
use monitaur_monitoring::MonitoringEngine;
use monitaur_network::NetworkIntelligenceEngine;
use monitaur_persistence::PersistenceEngine;
use monitaur_security::SecurityEngine;
use monitaur_visualization::VisualizationEngine;
use tracing::info;

use crate::api::create_router;
use crate::app_state::AppState;

#[derive(Parser)]
#[command(
    name = "monitaur",
    about = "Local-first infrastructure intelligence platform"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a one-shot analysis scan
    Scan {
        #[arg(long, default_value = "monitaur.db")]
        db: String,
    },
    /// Start the HTTP API server
    Serve {
        #[arg(long, default_value_t = 8080)]
        port: u16,
        #[arg(long, default_value = "monitaur.db")]
        db: String,
        /// Enable API authentication (optional, off by default)
        #[arg(long, default_value_t = false)]
        auth: bool,
    },
}

#[tokio::main]
async fn main() -> EngineResult<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Scan { db } => cmd_scan(&db).await,
        Commands::Serve { port, db, auth } => cmd_serve(port, &db, auth).await,
    }
}

// ── Scan ───────────────────────────────────────────────────────

async fn cmd_scan(db_path: &str) -> EngineResult<()> {
    info!("Monitaur v{} scan starting", env!("CARGO_PKG_VERSION"));

    let db = PersistenceEngine::open(db_path)?;
    let mut meta = MetadataEngine::new();

    // Discovery
    let discovery = DiscoveryEngine::new();
    let graph = discovery.discover().await?;
    meta.update(graph.clone());
    db.save_infra_graph(&graph)?;

    println!(
        "\n=== Discovery: {} services, {} networks, {} edges ===",
        graph.services.len(),
        graph.network_nodes.len(),
        graph.edges.len(),
    );

    // Monitoring
    let mut monitoring = MonitoringEngine::new().with_poll_interval(5);
    let snapshot = monitoring.snapshot(&graph.services).await?;
    db.save_metrics_snapshot(&snapshot)?;
    meta.snapshot_metrics(snapshot);

    // Security
    let security = SecurityEngine::new();
    let findings = security.analyze(&graph.services).await;
    for finding in &findings {
        db.save_finding(finding)?;
    }
    meta.snapshot_infra();

    println!("  Security: {} findings", findings.len());
    for f in &findings {
        println!("    [{:?}] {} — {}", f.severity, f.title, f.description);
    }

    // Network
    if let Ok(analysis) = NetworkIntelligenceEngine::new().analyze() {
        db.save_network_analysis(&analysis)?;
        println!("  Network: {} connections", analysis.connections.len());
    }

    // Visualization
    let topology = VisualizationEngine::new().render(&graph);
    println!(
        "  Topology: {} nodes, {} edges, {} groups",
        topology.nodes.len(),
        topology.edges.len(),
        topology.groups.len(),
    );

    let status = meta.status();
    println!(
        "  Metadata: {} cached, {} infra snapshots, {} metrics snapshots",
        status.services, status.infra_snapshots, status.metrics_snapshots,
    );

    println!("\nScan complete — all data written to {db_path}");
    Ok(())
}

// ── Serve ──────────────────────────────────────────────────────

async fn cmd_serve(port: u16, db_path: &str, auth: bool) -> EngineResult<()> {
    if auth {
        info!("API authentication enabled");
    }
    info!("Monitaur API server starting on port {port}");

    let state = AppState::new(db_path, auth)?;
    let app = create_router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("Listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| monitaur_core::error::EngineError::Io(format!("Failed to bind: {e}")))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| monitaur_core::error::EngineError::Io(format!("Server error: {e}")))?;

    Ok(())
}
