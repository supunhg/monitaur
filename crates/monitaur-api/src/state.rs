use std::sync::Arc;
use std::time::{Duration, Instant};

use monitaur_core::error::{EngineError, EngineResult};
use monitaur_core::models::InfraGraph;
use monitaur_discovery::DiscoveryEngine;
use monitaur_metadata::MetadataEngine;
use monitaur_monitoring::MonitoringEngine;
use monitaur_network::NetworkIntelligenceEngine;
use monitaur_persistence::PersistenceEngine;
use monitaur_security::SecurityEngine;
use monitaur_visualization::VisualizationEngine;
use tokio::sync::Mutex;
use tracing::info;

const CACHE_TTL: Duration = Duration::from_secs(30);

pub struct AppState {
    pub db: Mutex<PersistenceEngine>,
    pub monitoring: Mutex<MonitoringEngine>,
    pub metadata: Mutex<MetadataEngine>,
    pub auth_enabled: bool,
    cached_graph: Mutex<Option<(InfraGraph, Instant)>>,
}

impl AppState {
    pub fn new(db_path: &str, auth_enabled: bool) -> Result<Arc<Self>, EngineError> {
        let db = PersistenceEngine::open(db_path)?;
        Ok(Arc::new(Self {
            db: Mutex::new(db),
            monitoring: Mutex::new(MonitoringEngine::new().with_poll_interval(5)),
            metadata: Mutex::new(MetadataEngine::new()),
            auth_enabled,
            cached_graph: Mutex::new(None),
        }))
    }

    /// Returns the cached infra graph or runs a fresh discovery.
    /// Cache is valid for 30 seconds.
    pub async fn discover(self: &Arc<Self>) -> EngineResult<InfraGraph> {
        {
            let cache = self.cached_graph.lock().await;
            if let Some((graph, time)) = &*cache
                && time.elapsed() < CACHE_TTL
            {
                return Ok(graph.clone());
            }
        }

        info!("Cache miss — running infrastructure discovery");
        let discovery = DiscoveryEngine::new();
        let graph = discovery.discover().await?;

        // Update metadata
        {
            let mut meta = self.metadata.lock().await;
            meta.update(graph.clone());
        }

        // Persist
        {
            let db = self.db.lock().await;
            db.save_infra_graph(&graph).map_err(|e| {
                EngineError::Persistence(format!("Failed to persist discovery: {e}"))
            })?;
        }

        let mut cache = self.cached_graph.lock().await;
        *cache = Some((graph.clone(), Instant::now()));
        Ok(graph)
    }

    /// Force a fresh discovery, ignoring the cache.
    pub async fn force_discover(self: &Arc<Self>) -> EngineResult<InfraGraph> {
        info!("Forcing fresh infrastructure discovery");
        let discovery = DiscoveryEngine::new();
        let graph = discovery.discover().await?;

        {
            let mut meta = self.metadata.lock().await;
            meta.update(graph.clone());
        }
        {
            let db = self.db.lock().await;
            db.save_infra_graph(&graph).map_err(|e| {
                EngineError::Persistence(format!("Failed to persist discovery: {e}"))
            })?;
        }

        let mut cache = self.cached_graph.lock().await;
        *cache = Some((graph.clone(), Instant::now()));
        Ok(graph)
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
