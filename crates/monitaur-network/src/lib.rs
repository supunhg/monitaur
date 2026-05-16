pub mod classification;
pub mod dns_inspection;
pub mod traffic;

use std::collections::HashMap;

use monitaur_core::error::EngineResult;
use monitaur_core::network::{Connection, NetworkAnalysis, TrafficFlow};
use tracing::{info, warn};

#[derive(Default)]
pub struct NetworkIntelligenceEngine;

impl NetworkIntelligenceEngine {
    pub fn new() -> Self {
        Self
    }

    pub async fn analyze_connections(&self) -> EngineResult<Vec<Connection>> {
        info!("Reading active TCP connections");
        traffic::read_active_connections()
    }

    pub fn classify_flows(&self, connections: &[Connection]) -> Vec<TrafficFlow> {
        classification::build_traffic_flows(connections)
    }

    pub async fn analyze(&self) -> EngineResult<NetworkAnalysis> {
        let mut connections = traffic::read_active_connections()?;

        let container_pids: HashMap<String, Vec<u32>> = traffic::collect_container_pids().await;
        traffic::resolve_container_connections(&mut connections, &container_pids);

        let flows = classification::build_traffic_flows(&connections);
        let dns_queries = match dns_inspection::resolve_known_hosts() {
            Ok(queries) => queries,
            Err(error) => {
                warn!("Failed to resolve known hosts for network analysis: {error}");
                Vec::new()
            }
        };

        let dns_servers = match dns_inspection::read_resolv_conf() {
            Ok(servers) => servers,
            Err(error) => {
                warn!("Failed to read resolver configuration for network analysis: {error}");
                Vec::new()
            }
        };

        info!(
            "Network analysis: {} connections, {} flows, {} dns servers",
            connections.len(),
            flows.len(),
            dns_servers.len()
        );

        Ok(NetworkAnalysis {
            connections,
            flows,
            dns_queries,
        })
    }
}
