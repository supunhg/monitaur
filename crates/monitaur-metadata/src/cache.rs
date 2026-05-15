use std::sync::RwLock;

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

    pub fn set(&self, graph: InfraGraph) {
        let mut cached = self.graph.write().unwrap();
        info!(
            "Cache updated: {} services, {} nodes, {} edges",
            graph.services.len(),
            graph.network_nodes.len(),
            graph.edges.len(),
        );
        *cached = Some(graph);
    }

    pub fn get(&self) -> Option<InfraGraph> {
        self.graph.read().unwrap().clone()
    }

    pub fn services(&self) -> Vec<Service> {
        self.graph
            .read()
            .unwrap()
            .as_ref()
            .map(|g| g.services.clone())
            .unwrap_or_default()
    }

    pub fn network_nodes(&self) -> Vec<NetworkNode> {
        self.graph
            .read()
            .unwrap()
            .as_ref()
            .map(|g| g.network_nodes.clone())
            .unwrap_or_default()
    }

    pub fn edges(&self) -> Vec<Edge> {
        self.graph
            .read()
            .unwrap()
            .as_ref()
            .map(|g| g.edges.clone())
            .unwrap_or_default()
    }

    pub fn is_empty(&self) -> bool {
        self.graph.read().unwrap().is_none()
    }

    pub fn clear(&self) {
        *self.graph.write().unwrap() = None;
        info!("Cache cleared");
    }

    pub fn service_count(&self) -> usize {
        self.services().len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges().len()
    }
}
