use std::collections::HashMap;

use monitaur_core::models::{ExposureState, InfraGraph, ServiceClass};
use monitaur_core::visualization::NodeGroup;
use tracing::info;

#[derive(Default)]
pub struct NodeClusterer;

impl NodeClusterer {
    pub fn new() -> Self {
        Self
    }

    pub fn cluster_by_class(&self, graph: &InfraGraph) -> Vec<NodeGroup> {
        let mut class_map: HashMap<ServiceClass, Vec<String>> = HashMap::new();

        for service in &graph.services {
            class_map
                .entry(service.class.clone())
                .or_default()
                .push(service.id.clone());
        }

        let groups: Vec<NodeGroup> = class_map
            .into_iter()
            .map(|(class, node_ids)| NodeGroup {
                id: format!("class_{:?}", class),
                label: format!("{:?}", class),
                node_ids,
                group_type: "class".to_string(),
            })
            .collect();

        info!("Clustered {} service classes", groups.len());
        groups
    }

    pub fn cluster_by_network(&self, graph: &InfraGraph) -> Vec<NodeGroup> {
        let mut network_map: HashMap<String, Vec<String>> = HashMap::new();

        for service in &graph.services {
            for net in &service.networks {
                network_map
                    .entry(net.clone())
                    .or_default()
                    .push(service.id.clone());
            }
        }

        let groups: Vec<NodeGroup> = network_map
            .into_iter()
            .map(|(network, node_ids)| NodeGroup {
                id: format!("net_{network}"),
                label: format!("Network: {network}"),
                node_ids,
                group_type: "network".to_string(),
            })
            .collect();

        info!("Clustered {} networks", groups.len());
        groups
    }

    pub fn cluster_by_exposure(&self, graph: &InfraGraph) -> Vec<NodeGroup> {
        let mut exposed = Vec::new();
        let mut internal = Vec::new();

        for service in &graph.services {
            match service.exposure_state {
                ExposureState::Exposed => exposed.push(service.id.clone()),
                _ => internal.push(service.id.clone()),
            }
        }

        let mut groups = Vec::new();
        if !exposed.is_empty() {
            groups.push(NodeGroup {
                id: "exposure_exposed".to_string(),
                label: "Exposed".to_string(),
                node_ids: exposed,
                group_type: "exposure".to_string(),
            });
        }
        if !internal.is_empty() {
            groups.push(NodeGroup {
                id: "exposure_internal".to_string(),
                label: "Internal".to_string(),
                node_ids: internal,
                group_type: "exposure".to_string(),
            });
        }

        info!("Clustered by exposure: {} groups", groups.len());
        groups
    }

    pub fn all_clusters(&self, graph: &InfraGraph) -> Vec<NodeGroup> {
        let mut groups = Vec::new();
        groups.extend(self.cluster_by_class(graph));
        groups.extend(self.cluster_by_network(graph));
        groups.extend(self.cluster_by_exposure(graph));
        groups
    }
}
