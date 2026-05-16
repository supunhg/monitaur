pub mod migrations;
pub mod sqlite;

use monitaur_core::error::EngineResult;
use monitaur_core::metrics::MetricsSnapshot;
use monitaur_core::models::{InfraGraph, SecurityFinding, Severity};
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

    // ── Auth ──────────────────────────────────────────────────

    pub fn has_admin(&self) -> rusqlite::Result<bool> {
        self.store.has_admin()
    }

    pub fn set_password(&self, hash: &str) -> EngineResult<()> {
        self.store.set_password(hash)
    }

    pub fn get_password_hash(&self) -> Option<String> {
        self.store.get_password_hash()
    }

    pub fn create_token(&self, token: &str) -> EngineResult<()> {
        self.store.create_token(token)
    }

    pub fn validate_token(&self, token: &str) -> rusqlite::Result<bool> {
        self.store.validate_token(token)
    }

    pub fn cleanup_expired_tokens(&self) -> EngineResult<usize> {
        self.store.cleanup_expired_tokens()
    }

    // ── Historical reads ────────────────────────────────────────

    pub fn list_metrics_history(&self, limit: usize) -> EngineResult<Vec<MetricsSnapshot>> {
        self.store.list_metrics_history(limit)
    }

    pub fn list_findings(&self, limit: usize, severity: Option<Severity>) -> EngineResult<Vec<SecurityFinding>> {
        self.store.list_findings(limit, severity.map(|s| format!("{:?}", s)).as_deref())
    }
}
