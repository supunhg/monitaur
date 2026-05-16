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

        let remaining_node_ids: HashSet<&str> =
            filtered_nodes.iter().map(|node| node.id.as_str()).collect();
        let filtered_groups = graph
            .groups
            .iter()
            .map(|group| {
                let node_ids = group
                    .node_ids
                    .iter()
                    .filter(|id| remaining_node_ids.contains(id.as_str()))
                    .cloned()
                    .collect::<Vec<_>>();
                (group, node_ids)
            })
            .filter(|(_, node_ids)| !node_ids.is_empty())
            .map(
                |(group, node_ids)| monitaur_core::visualization::NodeGroup {
                    id: group.id.clone(),
                    label: group.label.clone(),
                    node_ids,
                    group_type: group.group_type.clone(),
                },
            )
            .collect();

        TopologyGraph {
            nodes: filtered_nodes,
            edges: graph.edges.clone(),
            groups: filtered_groups,
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

#[cfg(test)]
mod tests {
    use monitaur_core::visualization::{NodeGroup, TopologyEdge, TopologyGraph, TopologyNode};

    use super::GraphOptimizer;

    #[test]
    fn remove_isolated_also_prunes_group_membership() {
        let optimizer = GraphOptimizer::new();
        let graph = TopologyGraph {
            nodes: vec![
                TopologyNode {
                    id: "a".to_string(),
                    label: "A".to_string(),
                    node_type: "Service".to_string(),
                    group: "layer_1".to_string(),
                    layer: 1,
                    x: 0.0,
                    y: 0.0,
                    metadata: vec![],
                },
                TopologyNode {
                    id: "isolated".to_string(),
                    label: "Isolated".to_string(),
                    node_type: "Service".to_string(),
                    group: "layer_1".to_string(),
                    layer: 1,
                    x: 0.0,
                    y: 0.0,
                    metadata: vec![],
                },
            ],
            edges: vec![TopologyEdge {
                id: "e1".to_string(),
                source: "a".to_string(),
                target: "a".to_string(),
                label: "ConnectsTo".to_string(),
                edge_type: "ConnectsTo".to_string(),
                width: 1.0,
            }],
            groups: vec![NodeGroup {
                id: "layer_1".to_string(),
                label: "Layer".to_string(),
                node_ids: vec!["a".to_string(), "isolated".to_string()],
                group_type: "layer".to_string(),
            }],
            layers: vec!["Layer".to_string()],
        };

        let cleaned = optimizer.remove_isolated(&graph);

        assert_eq!(cleaned.nodes.len(), 1);
        assert_eq!(cleaned.groups.len(), 1);
        assert_eq!(cleaned.groups[0].node_ids, vec!["a".to_string()]);
    }
}
