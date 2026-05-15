pub mod clustering;
pub mod graph;
pub mod topology;

#[derive(Default)]
pub struct VisualizationEngine;

impl VisualizationEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self) {
        todo!("generate topology and graph visualizations")
    }
}
