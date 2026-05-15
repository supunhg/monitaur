use std::collections::VecDeque;
use std::time::SystemTime;

use monitaur_core::metrics::MetricsSnapshot;
use monitaur_core::models::InfraGraph;
use tracing::info;

const DEFAULT_MAX_SNAPSHOTS: usize = 10;

pub struct SnapshotManager {
    infra_snapshots: VecDeque<InfraSnapshot>,
    metrics_snapshots: VecDeque<MetricsSnapshot>,
    max_snapshots: usize,
}

pub struct InfraSnapshot {
    pub graph: InfraGraph,
    pub timestamp: SystemTime,
}

impl Default for SnapshotManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SnapshotManager {
    pub fn new() -> Self {
        Self {
            infra_snapshots: VecDeque::with_capacity(DEFAULT_MAX_SNAPSHOTS),
            metrics_snapshots: VecDeque::with_capacity(DEFAULT_MAX_SNAPSHOTS),
            max_snapshots: DEFAULT_MAX_SNAPSHOTS,
        }
    }

    pub fn with_capacity(max: usize) -> Self {
        Self {
            infra_snapshots: VecDeque::with_capacity(max),
            metrics_snapshots: VecDeque::with_capacity(max),
            max_snapshots: max,
        }
    }

    pub fn push_infra(&mut self, graph: InfraGraph) {
        if self.infra_snapshots.len() >= self.max_snapshots {
            self.infra_snapshots.pop_front();
        }
        self.infra_snapshots.push_back(InfraSnapshot {
            graph,
            timestamp: SystemTime::now(),
        });
        info!(
            "Infra snapshot stored ({} total)",
            self.infra_snapshots.len()
        );
    }

    pub fn push_metrics(&mut self, snapshot: MetricsSnapshot) {
        if self.metrics_snapshots.len() >= self.max_snapshots {
            self.metrics_snapshots.pop_front();
        }
        self.metrics_snapshots.push_back(snapshot);
        info!(
            "Metrics snapshot stored ({} total)",
            self.metrics_snapshots.len()
        );
    }

    pub fn latest_infra(&self) -> Option<&InfraSnapshot> {
        self.infra_snapshots.back()
    }

    pub fn latest_metrics(&self) -> Option<&MetricsSnapshot> {
        self.metrics_snapshots.back()
    }

    pub fn all_infra(&self) -> &VecDeque<InfraSnapshot> {
        &self.infra_snapshots
    }

    pub fn all_metrics(&self) -> &VecDeque<MetricsSnapshot> {
        &self.metrics_snapshots
    }

    pub fn infra_count(&self) -> usize {
        self.infra_snapshots.len()
    }

    pub fn metrics_count(&self) -> usize {
        self.metrics_snapshots.len()
    }
}
