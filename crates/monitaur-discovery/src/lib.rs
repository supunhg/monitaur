pub mod docker;
pub mod network;

use monitaur_core::error::EngineResult;
use monitaur_core::models::{InfraGraph, NetworkNode};
use tracing::info;

#[derive(Default)]
pub struct DiscoveryEngine;

impl DiscoveryEngine {
    pub fn new() -> Self {
        Self
    }

    pub async fn discover(&self) -> EngineResult<InfraGraph> {
        info!("Starting infrastructure discovery");

        let docker = docker::DockerDiscoverer::new();
        let net = network::NetworkDiscoverer::new();

        let services = docker.enumerate_containers().await?;
        let mut network_nodes = net.discover_interfaces().unwrap_or_default();

        let docker_networks = docker.enumerate_networks().await?;
        for (net_name, _containers) in &docker_networks {
            network_nodes.push(NetworkNode {
                id: format!("docker-net:{net_name}"),
                kind: monitaur_core::models::NetworkNodeKind::InternalService,
                addresses: vec![],
            });
        }

        let edges = docker.build_edges(&services).await?;

        info!(
            "Discovery complete: {} services, {} network nodes, {} edges",
            services.len(),
            network_nodes.len(),
            edges.len()
        );

        Ok(InfraGraph {
            services,
            network_nodes,
            edges,
        })
    }
}
