use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use monitaur_core::models::{Edge, InfraGraph, NetworkNode, Service};
use tracing::info;

#[derive(Default)]
pub struct EntityCache {
    graph: RwLock<Option<InfraGraph>>,
}

impl EntityCache {
    pub fn new() -> Self {
        Self {
            graph: RwLock::new(None),
        }
    }

    fn read_guard(&self) -> RwLockReadGuard<'_, Option<InfraGraph>> {
        self.graph.read().unwrap_or_else(|e| {
            tracing::warn!("Cache read lock poisoned, recovering");
            e.into_inner()
        })
    }

    fn write_guard(&self) -> RwLockWriteGuard<'_, Option<InfraGraph>> {
        self.graph.write().unwrap_or_else(|e| {
            tracing::warn!("Cache write lock poisoned, recovering");
            e.into_inner()
        })
    }

    pub fn set(&self, graph: InfraGraph) {
        *self.write_guard() = Some(graph);
        let g = self.read_guard();
        info!(
            "Cache updated: {} services, {} nodes, {} edges",
            g.as_ref().map(|g| g.services.len()).unwrap_or(0),
            g.as_ref().map(|g| g.network_nodes.len()).unwrap_or(0),
            g.as_ref().map(|g| g.edges.len()).unwrap_or(0),
        );
    }

    pub fn get(&self) -> Option<InfraGraph> {
        self.read_guard().clone()
    }

    pub fn services(&self) -> Vec<Service> {
        self.read_guard()
            .as_ref()
            .map(|g| g.services.clone())
            .unwrap_or_default()
    }

    pub fn network_nodes(&self) -> Vec<NetworkNode> {
        self.read_guard()
            .as_ref()
            .map(|g| g.network_nodes.clone())
            .unwrap_or_default()
    }

    pub fn edges(&self) -> Vec<Edge> {
        self.read_guard()
            .as_ref()
            .map(|g| g.edges.clone())
            .unwrap_or_default()
    }

    pub fn is_empty(&self) -> bool {
        self.read_guard().is_none()
    }

    pub fn clear(&self) {
        *self.write_guard() = None;
        info!("Cache cleared");
    }

    pub fn service_count(&self) -> usize {
        self.services().len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges().len()
    }
}
