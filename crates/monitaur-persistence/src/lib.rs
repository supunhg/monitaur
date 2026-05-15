pub mod migrations;
pub mod sqlite;

use monitaur_core::error::EngineResult;
use monitaur_core::metrics::MetricsSnapshot;
use monitaur_core::models::{InfraGraph, SecurityFinding};
use monitaur_core::network::NetworkAnalysis;
use sqlite::SqliteStore;

pub struct PersistenceEngine {
    store: SqliteStore,
}

impl PersistenceEngine {
    pub fn open(path: &str) -> EngineResult<Self> {
        let store = SqliteStore::open(path)?;
        Ok(Self { store })
    }

    pub fn save_infra_graph(&self, graph: &InfraGraph) -> EngineResult<()> {
        self.store.save_infra_graph(graph)
    }

    pub fn save_metrics_snapshot(&self, snapshot: &MetricsSnapshot) -> EngineResult<i64> {
        self.store.save_metrics_snapshot(snapshot)
    }

    pub fn save_network_analysis(&self, analysis: &NetworkAnalysis) -> EngineResult<()> {
        self.store.save_network_analysis(analysis)
    }

    pub fn save_finding(&self, finding: &SecurityFinding) -> EngineResult<()> {
        self.store.save_finding(finding)
    }
}
