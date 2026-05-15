pub mod classification;
pub mod dns_inspection;
pub mod traffic;

use std::collections::HashMap;

use monitaur_core::error::EngineResult;
use monitaur_core::network::{Connection, NetworkAnalysis, TrafficFlow};
use tracing::info;

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

    pub fn analyze(&self) -> EngineResult<NetworkAnalysis> {
        let mut connections = traffic::read_active_connections()?;

        // Build PID→container mapping from services
        // (container PIDs aren't directly available from `Service` model currently,
        //  so this is a placeholder for future enhancement)
        let container_pids: HashMap<String, Vec<u32>> = HashMap::new();
        traffic::resolve_container_connections(&mut connections, &container_pids);

        let flows = classification::build_traffic_flows(&connections);
        let dns_queries = dns_inspection::resolve_known_hosts().unwrap_or_default();

        let dns_servers = dns_inspection::read_resolv_conf().unwrap_or_default();

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
