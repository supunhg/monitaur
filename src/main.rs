use monitaur_core::error::EngineResult;
use monitaur_discovery::DiscoveryEngine;
use monitaur_monitoring::MonitoringEngine;
use tracing::info;

#[tokio::main]
async fn main() -> EngineResult<()> {
    tracing_subscriber::fmt::init();

    info!("Monitaur v{} starting", env!("CARGO_PKG_VERSION"));

    // ── Discovery ──────────────────────────────────────────────
    let discovery = DiscoveryEngine::new();
    let graph = discovery.discover().await?;

    println!("\n=== Discovery Results ===");
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

    // ── Monitoring ─────────────────────────────────────────────
    let mut monitoring = MonitoringEngine::new().with_poll_interval(5);

    println!("\n=== System Metrics (one-shot) ===");
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

    println!("\n  Container Metrics:");
    for cm in &snapshot.containers {
        println!(
            "    {} — CPU: {:.1}% Mem: {:.1}% Net: ↓{}/↑{}",
            &cm.container_id[..12],
            cm.cpu_percent,
            cm.memory_percent,
            bytes_to_human(cm.network_rx_bytes),
            bytes_to_human(cm.network_tx_bytes),
        );
    }

    println!("\nAll systems nominal.");
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
