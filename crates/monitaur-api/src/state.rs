use std::sync::Arc;

use monitaur_discovery::DiscoveryEngine;
use monitaur_metadata::MetadataEngine;
use monitaur_monitoring::MonitoringEngine;
use monitaur_network::NetworkIntelligenceEngine;
use monitaur_persistence::PersistenceEngine;
use monitaur_security::SecurityEngine;
use monitaur_visualization::VisualizationEngine;
use tokio::sync::Mutex;

pub struct AppState {
    pub db: Mutex<PersistenceEngine>,
    pub monitoring: Mutex<MonitoringEngine>,
    pub metadata: Mutex<MetadataEngine>,
    pub auth_enabled: bool,
}

impl AppState {
    pub fn new(db_path: &str, auth_enabled: bool) -> Result<Arc<Self>, monitaur_core::error::EngineError> {
        let db = PersistenceEngine::open(db_path)?;
        Ok(Arc::new(Self {
            db: Mutex::new(db),
            monitoring: Mutex::new(MonitoringEngine::new().with_poll_interval(5)),
            metadata: Mutex::new(MetadataEngine::new()),
            auth_enabled,
        }))
    }

    pub fn discovery(&self) -> DiscoveryEngine {
        DiscoveryEngine::new()
    }

    pub fn security(&self) -> SecurityEngine {
        SecurityEngine::new()
    }

    pub fn network(&self) -> NetworkIntelligenceEngine {
        NetworkIntelligenceEngine::new()
    }

    pub fn visualization(&self) -> VisualizationEngine {
        VisualizationEngine::new()
    }
}
