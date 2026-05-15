use std::path::Path;

use monitaur_core::error::{EngineError, EngineResult};
use monitaur_core::models::{NetworkNode, NetworkNodeKind};
use tracing::info;

#[derive(Default)]
pub struct NetworkDiscoverer;

impl NetworkDiscoverer {
    pub fn new() -> Self {
        Self
    }

    pub fn discover_interfaces(&self) -> EngineResult<Vec<NetworkNode>> {
        let mut nodes = Vec::new();

        let net_path = Path::new("/sys/class/net");
        if !net_path.exists() {
            return Err(EngineError::Discovery(
                "/sys/class/net not found".to_string(),
            ));
        }

        let entries = std::fs::read_dir(net_path)
            .map_err(|e| EngineError::Io(format!("Failed to read /sys/class/net: {e}")))?;

        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();

            if name == "lo" {
                continue;
            }

            let addr_path = entry.path().join("address");
            let mac = std::fs::read_to_string(&addr_path)
                .ok()
                .map(|s| s.trim().to_string())
                .unwrap_or_default();

            let mut addresses = Vec::new();
            if !mac.is_empty() {
                addresses.push(format!("mac:{mac}"));
            }

            nodes.push(NetworkNode {
                id: format!("iface:{name}"),
                kind: NetworkNodeKind::InternalService,
                addresses,
            });
        }

        info!("Discovered {} network interfaces", nodes.len());
        Ok(nodes)
    }
}
