use std::collections::HashMap;
use std::sync::Arc;

use monitaur_core::models::{Service, ServiceClass, ServiceType};
use tracing::info;

pub struct ServiceIndex {
    by_id: HashMap<String, Arc<Service>>,
    by_name: HashMap<String, Vec<Arc<Service>>>,
    by_class: HashMap<ServiceClass, Vec<Arc<Service>>>,
    by_type: HashMap<ServiceType, Vec<Arc<Service>>>,
    by_network: HashMap<String, Vec<Arc<Service>>>,
    by_port: HashMap<u16, Vec<Arc<Service>>>,
    exposed: Vec<Arc<Service>>,
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
            let svc = Arc::new(service.clone());

            self.by_id.insert(service.id.clone(), svc.clone());

            self.by_name
                .entry(service.name.clone())
                .or_default()
                .push(svc.clone());

            self.by_class
                .entry(service.class.clone())
                .or_default()
                .push(svc.clone());

            self.by_type
                .entry(service.service_type.clone())
                .or_default()
                .push(svc.clone());

            for net in &service.networks {
                self.by_network
                    .entry(net.clone())
                    .or_default()
                    .push(svc.clone());
            }

            for port in &service.ports {
                self.by_port
                    .entry(port.port)
                    .or_default()
                    .push(svc.clone());
            }

            if service.exposure_state == monitaur_core::models::ExposureState::Exposed {
                self.exposed.push(svc);
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

    pub fn by_id(&self, id: &str) -> Option<Arc<Service>> {
        self.by_id.get(id).cloned()
    }

    pub fn by_name(&self, name: &str) -> Vec<Arc<Service>> {
        self.by_name
            .get(name)
            .cloned()
            .unwrap_or_default()
    }

    pub fn by_class(&self, class: &ServiceClass) -> Vec<Arc<Service>> {
        self.by_class
            .get(class)
            .cloned()
            .unwrap_or_default()
    }

    pub fn by_type(&self, service_type: &ServiceType) -> Vec<Arc<Service>> {
        self.by_type
            .get(service_type)
            .cloned()
            .unwrap_or_default()
    }

    pub fn by_network(&self, network: &str) -> Vec<Arc<Service>> {
        self.by_network
            .get(network)
            .cloned()
            .unwrap_or_default()
    }

    pub fn by_port(&self, port: u16) -> Vec<Arc<Service>> {
        self.by_port
            .get(&port)
            .cloned()
            .unwrap_or_default()
    }

    pub fn exposed_services(&self) -> &[Arc<Service>] {
        &self.exposed
    }

    pub fn all_ids(&self) -> Vec<&str> {
        self.by_id.keys().map(|s| s.as_str()).collect()
    }

    pub fn count(&self) -> usize {
        self.by_id.len()
    }
}
