use std::collections::HashMap;

use monitaur_core::models::{Service, ServiceClass, ServiceType};
use tracing::info;

pub struct ServiceIndex {
    by_id: HashMap<String, Service>,
    by_name: HashMap<String, Vec<Service>>,
    by_class: HashMap<ServiceClass, Vec<Service>>,
    by_type: HashMap<ServiceType, Vec<Service>>,
    by_network: HashMap<String, Vec<Service>>,
    by_port: HashMap<u16, Vec<Service>>,
    exposed: Vec<Service>,
}

impl Default for ServiceIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceIndex {
    pub fn new() -> Self {
        Self {
            by_id: HashMap::new(),
            by_name: HashMap::new(),
            by_class: HashMap::new(),
            by_type: HashMap::new(),
            by_network: HashMap::new(),
            by_port: HashMap::new(),
            exposed: Vec::new(),
        }
    }

    pub fn rebuild(&mut self, services: &[Service]) {
        self.by_id.clear();
        self.by_name.clear();
        self.by_class.clear();
        self.by_type.clear();
        self.by_network.clear();
        self.by_port.clear();
        self.exposed.clear();

        for service in services {
            // By ID (unique)
            self.by_id.insert(service.id.clone(), service.clone());

            // By name
            self.by_name
                .entry(service.name.clone())
                .or_default()
                .push(service.clone());

            // By class
            self.by_class
                .entry(service.class.clone())
                .or_default()
                .push(service.clone());

            // By type
            self.by_type
                .entry(service.service_type.clone())
                .or_default()
                .push(service.clone());

            // By network
            for net in &service.networks {
                self.by_network
                    .entry(net.clone())
                    .or_default()
                    .push(service.clone());
            }

            // By port
            for port in &service.ports {
                self.by_port
                    .entry(port.port)
                    .or_default()
                    .push(service.clone());
            }

            // Exposed services
            if service.exposure_state == monitaur_core::models::ExposureState::Exposed {
                self.exposed.push(service.clone());
            }
        }

        info!(
            "Index rebuilt: {} by_id, {} by_name, {} by_class, {} by_network, {} by_port, {} exposed",
            self.by_id.len(),
            self.by_name.len(),
            self.by_class.len(),
            self.by_network.len(),
            self.by_port.len(),
            self.exposed.len(),
        );
    }

    pub fn by_id(&self, id: &str) -> Option<&Service> {
        self.by_id.get(id)
    }

    pub fn by_name(&self, name: &str) -> Vec<&Service> {
        self.by_name
            .get(name)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn by_class(&self, class: &ServiceClass) -> Vec<&Service> {
        self.by_class
            .get(class)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn by_type(&self, service_type: &ServiceType) -> Vec<&Service> {
        self.by_type
            .get(service_type)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn by_network(&self, network: &str) -> Vec<&Service> {
        self.by_network
            .get(network)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn by_port(&self, port: u16) -> Vec<&Service> {
        self.by_port
            .get(&port)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn exposed_services(&self) -> &[Service] {
        &self.exposed
    }

    pub fn all_ids(&self) -> Vec<&str> {
        self.by_id.keys().map(|s| s.as_str()).collect()
    }

    pub fn count(&self) -> usize {
        self.by_id.len()
    }
}
