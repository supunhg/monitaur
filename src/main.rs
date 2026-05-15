use monitaur_core::error::EngineResult;
use monitaur_discovery::DiscoveryEngine;
use tracing::info;

#[tokio::main]
async fn main() -> EngineResult<()> {
    tracing_subscriber::fmt::init();

    info!("Monitaur v{} starting", env!("CARGO_PKG_VERSION"));

    let discovery = DiscoveryEngine::new();
    let graph = discovery.discover().await?;

    println!("\n=== Discovery Results ===");
    println!(
        "Services: {} | Networks: {} | Edges: {}",
        graph.services.len(),
        graph.network_nodes.len(),
        graph.edges.len()
    );
    println!();

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
        println!();
    }

    Ok(())
}
