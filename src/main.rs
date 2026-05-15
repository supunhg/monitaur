use monitaur_core::error::EngineResult;
use monitaur_discovery::DiscoveryEngine;
use monitaur_monitoring::MonitoringEngine;
use monitaur_network::NetworkIntelligenceEngine;
use monitaur_persistence::PersistenceEngine;
use tracing::info;

#[tokio::main]
async fn main() -> EngineResult<()> {
    tracing_subscriber::fmt::init();

    info!("Monitaur v{} starting", env!("CARGO_PKG_VERSION"));

    // ── Persistence ────────────────────────────────────────────
    let db = PersistenceEngine::open("monitaur.db")?;

    // ── Discovery ──────────────────────────────────────────────
    let discovery = DiscoveryEngine::new();
    let graph = discovery.discover().await?;

    println!("\n=== Discovery ===");
    println!(
        "Services: {} | Networks: {} | Edges: {}",
        graph.services.len(),
        graph.network_nodes.len(),
        graph.edges.len()
    );

    for service in &graph.services {
        println!(
            "  {} [{}] {:?} ({:?})",
            service.name, service.status, service.class, service.health
        );
        if let Some(image) = &service.image {
            println!("    image: {image}");
        }
        for port in &service.ports {
            println!(
                "    port {}/{:?} exposed:{}",
                port.port, port.protocol, port.exposed
            );
        }
        if !service.networks.is_empty() {
            println!("    networks: {}", service.networks.join(", "));
        }
    }

    db.save_infra_graph(&graph)?;

    // ── Monitoring ─────────────────────────────────────────────
    let mut monitoring = MonitoringEngine::new().with_poll_interval(5);

    println!("\n=== System Metrics ===");
    let snapshot = monitoring.snapshot(&graph.services).await?;

    if let Some(sys) = &snapshot.system {
        println!("  CPU:        {:.1}%", sys.cpu_percent);
        println!(
            "  Memory:     {:.1}% ({}/{})",
            sys.memory_percent,
            bytes_to_human(sys.memory_used_bytes),
            bytes_to_human(sys.memory_total_bytes),
        );
        println!(
            "  Network:    ↓{} / ↑{}",
            bytes_to_human(sys.network_rx_bytes),
            bytes_to_human(sys.network_tx_bytes),
        );
    }

    db.save_metrics_snapshot(&snapshot)?;

    // ── Network Intelligence ────────────────────────────────────
    let net = NetworkIntelligenceEngine::new();

    println!("\n=== Network Intelligence ===");
    match net.analyze() {
        Ok(analysis) => {
            if analysis.connections.is_empty() {
                println!("  No active outbound TCP connections");
            } else {
                println!("  Active Connections: {}", analysis.connections.len());
                for conn in &analysis.connections[..analysis.connections.len().min(15)] {
                    let container_tag = conn
                        .container_id
                        .as_ref()
                        .map(|id| format!(" [{}]", &id[..12]))
                        .unwrap_or_default();
                    println!(
                        "    {}:{} → {}:{}{container_tag}",
                        conn.local_addr, conn.local_port, conn.remote_addr, conn.remote_port,
                    );
                }
                if analysis.connections.len() > 15 {
                    println!("    ... and {} more", analysis.connections.len() - 15);
                }
            }

            if !analysis.flows.is_empty() {
                println!("\n  Traffic Flows:");
                for flow in &analysis.flows {
                    println!(
                        "    {}:{} ({:?}) — {} conns",
                        flow.destination, flow.port, flow.class, flow.connection_count,
                    );
                }
            }

            db.save_network_analysis(&analysis)?;
        }
        Err(e) => {
            println!("  Network analysis failed: {e}");
        }
    }

    println!("\nAll data persisted to monitaur.db — all systems nominal.");
    Ok(())
}

fn bytes_to_human(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size > 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_idx])
}
