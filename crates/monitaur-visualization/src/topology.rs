use std::collections::{HashMap, HashSet};

use monitaur_core::models::{InfraGraph, ServiceClass};
use monitaur_core::visualization::{TopologyEdge, TopologyGraph, TopologyNode};
use tracing::info;

const LAYER_NAMES: &[&str] = &[
    "Internet",
    "Reverse Proxy",
    "Web App",
    "Worker / API",
    "Data Store",
    "Infrastructure",
];

fn classify_layer(class: &ServiceClass) -> usize {
    match class {
        ServiceClass::ReverseProxy => 1,
        ServiceClass::WebApp => 2,
        ServiceClass::Worker | ServiceClass::Messaging => 3,
        ServiceClass::Database | ServiceClass::Cache => 4,
        _ => 5, // Unknown, Utility, Monitoring, Security
    }
}

#[derive(Default)]
pub struct TopologyGenerator;

impl TopologyGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(&self, graph: &InfraGraph) -> TopologyGraph {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut groups: Vec<monitaur_core::visualization::NodeGroup> = Vec::new();

        // Track how many nodes per layer for vertical spacing
        let mut layer_counts: HashMap<usize, usize> = HashMap::new();
        let mut layer_heights: HashMap<usize, f64> = HashMap::new();

        // First pass: count nodes per layer
        for service in &graph.services {
            let layer = classify_layer(&service.class);
            *layer_counts.entry(layer).or_insert(0) += 1;
        }

        // Second pass: create positioned nodes
        let mut layer_positions: HashMap<usize, f64> = HashMap::new();
        let spacing_x = 200.0;
        let spacing_y = 150.0;

        for service in &graph.services {
            let layer = classify_layer(&service.class);
            let count = layer_counts.get(&layer).copied().unwrap_or(1) as f64;
            let index = layer_positions.get(&layer).copied().unwrap_or(0.0);

            let x = (index - (count - 1.0) / 2.0) * spacing_x;
            let y = layer as f64 * spacing_y + 50.0;

            let mut metadata = Vec::new();
            metadata.push(("status".to_string(), service.status.clone()));
            metadata.push(("health".to_string(), format!("{:?}", service.health)));
            metadata.push((
                "exposure".to_string(),
                format!("{:?}", service.exposure_state),
            ));
            if let Some(ref image) = service.image {
                metadata.push(("image".to_string(), image.clone()));
            }

            nodes.push(TopologyNode {
                id: service.id.clone(),
                label: service.name.clone(),
                node_type: format!("{:?}", service.class),
                group: format!("layer_{layer}"),
                layer,
                x,
                y,
                metadata,
            });

            layer_positions.insert(layer, index + 1.0);
            layer_heights.insert(layer, y + spacing_y);
        }

        // Network nodes
        for node in &graph.network_nodes {
            let y = 6.0 * spacing_y + 50.0;
            let idx = layer_positions.get(&6).copied().unwrap_or(0.0);

            nodes.push(TopologyNode {
                id: node.id.clone(),
                label: node.id.clone(),
                node_type: format!("{:?}", node.kind),
                group: "layer_6".to_string(),
                layer: 6,
                x: idx * spacing_x - 200.0,
                y,
                metadata: vec![],
            });

            layer_positions.insert(6, idx + 1.0);
        }

        // Edges — only include edges whose endpoints both exist as nodes
        let node_ids: HashSet<&str> = nodes.iter().map(|n| n.id.as_str()).collect();
        for edge in &graph.edges {
            if !node_ids.contains(edge.source_id.as_str()) || !node_ids.contains(edge.target_id.as_str()) {
                continue;
            }
            edges.push(TopologyEdge {
                id: format!("e_{}_{}", edge.source_id, edge.target_id),
                source: edge.source_id.clone(),
                target: edge.target_id.clone(),
                label: format!("{:?}", edge.relation),
                edge_type: format!("{:?}", edge.relation),
                width: 1.5,
            });
        }

        // Build groups by layer
        let mut group_map: HashMap<usize, Vec<String>> = HashMap::new();
        for node in &nodes {
            group_map
                .entry(node.layer)
                .or_default()
                .push(node.id.clone());
        }

        for (layer, node_ids) in group_map {
            groups.push(monitaur_core::visualization::NodeGroup {
                id: format!("layer_{layer}"),
                label: LAYER_NAMES.get(layer).unwrap_or(&"Unknown").to_string(),
                node_ids,
                group_type: "layer".to_string(),
            });
        }

        let layers: Vec<String> = LAYER_NAMES.iter().map(|s| s.to_string()).collect();

        info!(
            "Topology generated: {} nodes, {} edges, {} groups",
            nodes.len(),
            edges.len(),
            groups.len()
        );

        TopologyGraph {
            nodes,
            edges,
            groups,
            layers,
        }
    }
}
