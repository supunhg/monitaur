use std::collections::HashSet;

use monitaur_core::visualization::{TopologyEdge, TopologyGraph, TopologyNode};
use tracing::info;

#[derive(Default)]
pub struct GraphOptimizer;

impl GraphOptimizer {
    pub fn new() -> Self {
        Self
    }

    /// Removes duplicate edges (same source, target, and relation).
    pub fn deduplicate_edges(&self, graph: &TopologyGraph) -> Vec<TopologyEdge> {
        let mut seen: HashSet<(String, String, String)> = HashSet::new();
        let mut deduped = Vec::new();

        for edge in &graph.edges {
            let key = (
                edge.source.clone(),
                edge.target.clone(),
                edge.edge_type.clone(),
            );
            if seen.insert(key) {
                deduped.push(edge.clone());
            }
        }

        if deduped.len() < graph.edges.len() {
            info!("Deduplicated {} edges", graph.edges.len() - deduped.len());
        }

        deduped
    }

    /// Removes nodes that have no connections (isolated nodes).
    pub fn remove_isolated(&self, graph: &TopologyGraph) -> TopologyGraph {
        let connected: HashSet<&str> = graph
            .edges
            .iter()
            .flat_map(|e| [e.source.as_str(), e.target.as_str()])
            .collect();

        let filtered_nodes: Vec<TopologyNode> = graph
            .nodes
            .iter()
            .filter(|n| connected.contains(n.id.as_str()))
            .cloned()
            .collect();

        let removed = graph.nodes.len() - filtered_nodes.len();
        if removed > 0 {
            info!("Removed {removed} isolated nodes");
        }

        TopologyGraph {
            nodes: filtered_nodes,
            edges: graph.edges.clone(),
            groups: graph.groups.clone(),
            layers: graph.layers.clone(),
        }
    }

    /// Assigns visual weight to edges based on relation type.
    pub fn weigh_edges(&self, graph: &TopologyGraph) -> Vec<TopologyEdge> {
        graph
            .edges
            .iter()
            .map(|e| {
                let width = match e.edge_type.as_str() {
                    "Exposes" => 3.0,
                    "ConnectsTo" => 2.0,
                    "DependsOn" => 2.5,
                    _ => 1.0,
                };
                TopologyEdge { width, ..e.clone() }
            })
            .collect()
    }

    /// Full optimization pipeline.
    pub fn optimize(&self, graph: &mut TopologyGraph) {
        let deduped = self.deduplicate_edges(graph);
        graph.edges = deduped;

        let weighed = self.weigh_edges(graph);
        graph.edges = weighed;

        let cleaned = self.remove_isolated(graph);
        graph.nodes = cleaned.nodes;
        graph.edges = cleaned.edges;

        info!(
            "Graph optimized: {} nodes, {} edges",
            graph.nodes.len(),
            graph.edges.len()
        );
    }
}
