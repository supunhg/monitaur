pub mod clustering;
pub mod graph;
pub mod topology;

use clustering::NodeClusterer;
use graph::GraphOptimizer;
use monitaur_core::models::InfraGraph;
use monitaur_core::visualization::TopologyGraph;
use topology::TopologyGenerator;
use tracing::info;

#[derive(Default)]
pub struct VisualizationEngine;

impl VisualizationEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, graph: &InfraGraph) -> TopologyGraph {
        info!("Generating topology visualization");

        let generator = TopologyGenerator::new();
        let mut topology = generator.generate(graph);

        let clusterer = NodeClusterer::new();
        let clusters = clusterer.all_clusters(graph);
        topology.groups.extend(clusters);

        let optimizer = GraphOptimizer::new();
        optimizer.optimize(&mut topology);

        info!(
            "Visualization complete: {} nodes, {} edges, {} groups",
            topology.nodes.len(),
            topology.edges.len(),
            topology.groups.len(),
        );

        topology
    }
}
