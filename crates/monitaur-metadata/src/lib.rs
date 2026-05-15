pub mod cache;
pub mod indexing;
pub mod snapshots;

use cache::EntityCache;
use indexing::ServiceIndex;
use monitaur_core::metrics::MetricsSnapshot;
use monitaur_core::models::InfraGraph;
use snapshots::SnapshotManager;
use tracing::info;

pub struct MetadataEngine {
    pub cache: EntityCache,
    pub index: ServiceIndex,
    pub snapshots: SnapshotManager,
}

impl Default for MetadataEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl MetadataEngine {
    pub fn new() -> Self {
        Self {
            cache: EntityCache::new(),
            index: ServiceIndex::new(),
            snapshots: SnapshotManager::new(),
        }
    }

    pub fn with_snapshot_capacity(capacity: usize) -> Self {
        Self {
            cache: EntityCache::new(),
            index: ServiceIndex::new(),
            snapshots: SnapshotManager::with_capacity(capacity),
        }
    }

    pub fn update(&mut self, graph: InfraGraph) {
        let services = graph.services.clone();
        self.cache.set(graph);
        self.index.rebuild(&services);
        info!("Metadata engine updated");
    }

    pub fn snapshot_infra(&mut self) {
        if let Some(graph) = self.cache.get() {
            self.snapshots.push_infra(graph);
        }
    }

    pub fn snapshot_metrics(&mut self, snapshot: MetricsSnapshot) {
        self.snapshots.push_metrics(snapshot);
    }

    pub fn status(&self) -> MetadataStatus {
        MetadataStatus {
            services: self.cache.service_count(),
            edges: self.cache.edge_count(),
            indexed: self.index.count(),
            infra_snapshots: self.snapshots.infra_count(),
            metrics_snapshots: self.snapshots.metrics_count(),
        }
    }
}

pub struct MetadataStatus {
    pub services: usize,
    pub edges: usize,
    pub indexed: usize,
    pub infra_snapshots: usize,
    pub metrics_snapshots: usize,
}
