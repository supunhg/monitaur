use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TopologyNode {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub group: String,
    pub layer: usize,
    pub x: f64,
    pub y: f64,
    pub metadata: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TopologyEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: String,
    pub edge_type: String,
    pub width: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeGroup {
    pub id: String,
    pub label: String,
    pub node_ids: Vec<String>,
    pub group_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TopologyGraph {
    pub nodes: Vec<TopologyNode>,
    pub edges: Vec<TopologyEdge>,
    pub groups: Vec<NodeGroup>,
    pub layers: Vec<String>,
}
