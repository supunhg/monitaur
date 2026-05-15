use std::collections::HashSet;

use bollard::Docker;
use bollard::container::ListContainersOptions;
use bollard::models::ContainerSummary;
use monitaur_core::error::{EngineError, EngineResult};
use monitaur_core::models::{
    Edge, ExposureState, Health, Port, Protocol, RelationType, Service, ServiceClass, ServiceType,
};
use tracing::info;

const PROXY_IMAGES: &[&str] = &[
    "nginx",
    "traefik",
    "caddy",
    "haproxy",
    "envoy",
    "httpd",
    "apache",
    "kong",
    "ambassador",
    "socat",
];

const DATABASE_IMAGES: &[&str] = &[
    "postgres",
    "mysql",
    "mariadb",
    "mongo",
    "redis",
    "cassandra",
    "cockroachdb",
    "sqlite",
    "couchdb",
    "neo4j",
    "influxdb",
    "timescaledb",
];

const MESSAGING_IMAGES: &[&str] = &[
    "rabbitmq",
    "kafka",
    "nats",
    "pulsar",
    "mosquitto",
    "emqx",
    "vernemq",
];

#[derive(Default)]
pub struct DockerDiscoverer;

impl DockerDiscoverer {
    pub fn new() -> Self {
        Self
    }

    pub async fn enumerate_containers(&self) -> EngineResult<Vec<Service>> {
        let docker = Docker::connect_with_local_defaults()
            .map_err(|e| EngineError::Discovery(format!("Failed to connect to Docker: {e}")))?;

        let options = ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        };

        let containers = docker
            .list_containers(Some(options))
            .await
            .map_err(|e| EngineError::Discovery(format!("Failed to list containers: {e}")))?;

        if containers.is_empty() {
            info!("No containers found");
            return Ok(Vec::new());
        }

        let services: Vec<Service> = containers
            .into_iter()
            .map(Self::container_to_service)
            .collect();

        info!("Discovered {} containers", services.len());
        Ok(services)
    }

    fn container_to_service(container: ContainerSummary) -> Service {
        let id = container.id.unwrap_or_default();
        let name = container
            .names
            .as_ref()
            .and_then(|names| names.first().cloned())
            .unwrap_or_else(|| id.clone())
            .trim_start_matches('/')
            .to_string();

        let image = container.image.clone().unwrap_or_default();
        let state = container.state.unwrap_or_default();

        let health = match state.as_str() {
            "running" => Health::Healthy,
            "restarting" | "paused" => Health::Degraded,
            "exited" | "dead" => Health::Unhealthy,
            _ => Health::Unknown,
        };

        let ports = Self::dedup_ports(container.ports.unwrap_or_default());
        let labels = container.labels.unwrap_or_default();
        let networks: Vec<String> = container
            .network_settings
            .as_ref()
            .and_then(|ns| ns.networks.as_ref())
            .map(|nets| nets.keys().cloned().collect())
            .unwrap_or_default();

        let class = classify_image(&image);
        let exposure_state = if ports.iter().any(|p| p.exposed) {
            ExposureState::Exposed
        } else {
            ExposureState::Internal
        };

        Service {
            id,
            name,
            image: Some(image),
            service_type: ServiceType::Container,
            class,
            ports,
            networks,
            health,
            status: state,
            labels,
            exposure_state,
        }
    }

    fn dedup_ports(ports: Vec<bollard::models::Port>) -> Vec<Port> {
        let mut seen: HashSet<(u16, String)> = HashSet::new();
        ports
            .into_iter()
            .filter(|p| {
                let proto = match p.typ.as_ref().map(|t| t.to_string()).as_deref() {
                    Some("udp") => "udp".to_string(),
                    _ => "tcp".to_string(),
                };
                let key = (p.public_port.unwrap_or(p.private_port), proto);
                seen.insert(key)
            })
            .map(|p| Port {
                port: p.public_port.unwrap_or(p.private_port),
                protocol: match p.typ.as_ref().map(|t| t.to_string()).as_deref() {
                    Some("udp") => Protocol::Udp,
                    _ => Protocol::Tcp,
                },
                exposed: p.public_port.is_some(),
            })
            .collect()
    }

    pub async fn enumerate_networks(&self) -> EngineResult<Vec<(String, Vec<String>)>> {
        let docker = Docker::connect_with_local_defaults()
            .map_err(|e| EngineError::Discovery(format!("Failed to connect to Docker: {e}")))?;

        let networks = docker
            .list_networks::<String>(None)
            .await
            .map_err(|e| EngineError::Discovery(format!("Failed to list networks: {e}")))?;

        Ok(networks
            .into_iter()
            .filter_map(|n| {
                let name = n.name?;
                let containers: Vec<String> =
                    n.containers.unwrap_or_default().into_keys().collect();
                Some((name, containers))
            })
            .collect())
    }

    pub async fn build_edges(&self, services: &[Service]) -> EngineResult<Vec<Edge>> {
        let mut edges = Vec::new();

        for service in services {
            for net_name in &service.networks {
                edges.push(Edge {
                    source_id: service.id.clone(),
                    target_id: format!("docker-net:{net_name}"),
                    relation: RelationType::ConnectsTo,
                });
            }

            for port in &service.ports {
                if port.exposed {
                    edges.push(Edge {
                        source_id: service.id.clone(),
                        target_id: format!("host:{}", port.port),
                        relation: RelationType::Exposes,
                    });
                }
            }
        }

        Ok(edges)
    }
}

fn classify_image(image: &str) -> ServiceClass {
    let lower = image.to_lowercase();

    if PROXY_IMAGES.iter().any(|p| lower.contains(p)) {
        return ServiceClass::ReverseProxy;
    }
    if DATABASE_IMAGES.iter().any(|d| lower.contains(d)) {
        return ServiceClass::Database;
    }
    if MESSAGING_IMAGES.iter().any(|m| lower.contains(m)) {
        return ServiceClass::Messaging;
    }
    if lower.contains("worker") || lower.contains("celery") {
        return ServiceClass::Worker;
    }
    if lower.contains("prometheus")
        || lower.contains("grafana")
        || lower.contains("datadog")
        || lower.contains("statsd")
    {
        return ServiceClass::Monitoring;
    }

    ServiceClass::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;
    use bollard::models::Port;

    fn make_port(public: Option<u16>, private: u16, proto: &str) -> Port {
        Port {
            ip: None,
            private_port: private,
            public_port: public,
            typ: Some(match proto {
                "udp" => bollard::models::PortTypeEnum::UDP,
                _ => bollard::models::PortTypeEnum::TCP,
            }),
        }
    }

    #[test]
    fn test_dedup_ports() {
        let ports = vec![
            make_port(Some(8080), 80, "tcp"),
            make_port(Some(8080), 80, "tcp"),
            make_port(Some(8443), 443, "tcp"),
            make_port(None, 5432, "tcp"),
        ];

        let deduped = DockerDiscoverer::dedup_ports(ports);
        assert_eq!(deduped.len(), 3);
    }

    #[test]
    fn test_dedup_ports_no_public() {
        let ports = vec![
            make_port(None, 80, "tcp"),
            make_port(None, 80, "tcp"),
            make_port(None, 443, "tcp"),
        ];

        let deduped = DockerDiscoverer::dedup_ports(ports);
        assert_eq!(deduped.len(), 2);
    }

    #[test]
    fn test_classify_image_database() {
        assert_eq!(classify_image("postgres:16-alpine"), ServiceClass::Database);
        assert_eq!(classify_image("redis:7-alpine"), ServiceClass::Database);
        assert_eq!(classify_image("mysql:8"), ServiceClass::Database);
    }

    #[test]
    fn test_classify_image_reverse_proxy() {
        assert_eq!(classify_image("nginx:latest"), ServiceClass::ReverseProxy);
        assert_eq!(classify_image("traefik:v3"), ServiceClass::ReverseProxy);
        assert_eq!(classify_image("caddy:alpine"), ServiceClass::ReverseProxy);
    }

    #[test]
    fn test_classify_image_messaging() {
        assert_eq!(classify_image("rabbitmq:3"), ServiceClass::Messaging);
        assert_eq!(classify_image("kafka:latest"), ServiceClass::Messaging);
    }

    #[test]
    fn test_classify_image_monitoring() {
        assert_eq!(classify_image("prom/prometheus"), ServiceClass::Monitoring);
        assert_eq!(classify_image("grafana/grafana"), ServiceClass::Monitoring);
    }

    #[test]
    fn test_classify_image_worker() {
        assert_eq!(classify_image("myapp-worker:latest"), ServiceClass::Worker);
    }

    #[test]
    fn test_classify_image_unknown() {
        assert_eq!(classify_image("ubuntu:latest"), ServiceClass::Unknown);
    }

    #[test]
    fn test_dedup_with_protocol_separation() {
        let ports = vec![
            make_port(Some(53), 53, "tcp"),
            make_port(Some(53), 53, "udp"),
        ];

        let deduped = DockerDiscoverer::dedup_ports(ports);
        assert_eq!(deduped.len(), 2);
    }
}
