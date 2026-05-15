use monitaur_core::models::{Protocol, SecurityFinding, Service, Severity};
use std::time::SystemTime;
use uuid::Uuid;

/// Risk levels for exposed ports
#[derive(Debug, Clone, PartialEq)]
pub enum PortRisk {
    Critical,
    High,
    Medium,
    Low,
}

/// Sensitive port definitions
struct SensitivePort {
    port: u16,
    protocol: Protocol,
    risk: PortRisk,
    service: &'static str,
    description: &'static str,
}

const SENSITIVE_PORTS: &[SensitivePort] = &[
    // Critical — remote code execution / admin interfaces
    SensitivePort {
        port: 22,
        protocol: Protocol::Tcp,
        risk: PortRisk::Critical,
        service: "SSH",
        description: "SSH remote administration",
    },
    SensitivePort {
        port: 23,
        protocol: Protocol::Tcp,
        risk: PortRisk::Critical,
        service: "Telnet",
        description: "Unencrypted remote access",
    },
    SensitivePort {
        port: 3389,
        protocol: Protocol::Tcp,
        risk: PortRisk::Critical,
        service: "RDP",
        description: "Remote Desktop Protocol",
    },
    SensitivePort {
        port: 5900,
        protocol: Protocol::Tcp,
        risk: PortRisk::Critical,
        service: "VNC",
        description: "VNC remote access",
    },
    SensitivePort {
        port: 2375,
        protocol: Protocol::Tcp,
        risk: PortRisk::Critical,
        service: "Docker API",
        description: "Unencrypted Docker daemon socket",
    },
    // High — databases, message queues, management
    SensitivePort {
        port: 5432,
        protocol: Protocol::Tcp,
        risk: PortRisk::High,
        service: "PostgreSQL",
        description: "PostgreSQL database",
    },
    SensitivePort {
        port: 3306,
        protocol: Protocol::Tcp,
        risk: PortRisk::High,
        service: "MySQL",
        description: "MySQL/MariaDB database",
    },
    SensitivePort {
        port: 27017,
        protocol: Protocol::Tcp,
        risk: PortRisk::High,
        service: "MongoDB",
        description: "MongoDB database",
    },
    SensitivePort {
        port: 6379,
        protocol: Protocol::Tcp,
        risk: PortRisk::High,
        service: "Redis",
        description: "Redis cache",
    },
    SensitivePort {
        port: 9200,
        protocol: Protocol::Tcp,
        risk: PortRisk::High,
        service: "Elasticsearch",
        description: "Elasticsearch HTTP API",
    },
    SensitivePort {
        port: 9092,
        protocol: Protocol::Tcp,
        risk: PortRisk::High,
        service: "Kafka",
        description: "Apache Kafka broker",
    },
    SensitivePort {
        port: 5672,
        protocol: Protocol::Tcp,
        risk: PortRisk::High,
        service: "RabbitMQ",
        description: "RabbitMQ AMQP",
    },
    SensitivePort {
        port: 15672,
        protocol: Protocol::Tcp,
        risk: PortRisk::High,
        service: "RabbitMQ Management",
        description: "RabbitMQ management UI",
    },
    SensitivePort {
        port: 8443,
        protocol: Protocol::Tcp,
        risk: PortRisk::High,
        service: "HTTPS Alt",
        description: "Alternative HTTPS port",
    },
    // Medium — web services, monitoring
    SensitivePort {
        port: 80,
        protocol: Protocol::Tcp,
        risk: PortRisk::Medium,
        service: "HTTP",
        description: "Unencrypted HTTP",
    },
    SensitivePort {
        port: 8080,
        protocol: Protocol::Tcp,
        risk: PortRisk::Medium,
        service: "HTTP Proxy",
        description: "HTTP proxy/alternative",
    },
    SensitivePort {
        port: 3000,
        protocol: Protocol::Tcp,
        risk: PortRisk::Medium,
        service: "Web App",
        description: "Web application",
    },
    SensitivePort {
        port: 4000,
        protocol: Protocol::Tcp,
        risk: PortRisk::Medium,
        service: "Web App",
        description: "Web application",
    },
    SensitivePort {
        port: 5000,
        protocol: Protocol::Tcp,
        risk: PortRisk::Medium,
        service: "Web App",
        description: "Web application",
    },
    SensitivePort {
        port: 9000,
        protocol: Protocol::Tcp,
        risk: PortRisk::Medium,
        service: "MinIO",
        description: "MinIO/S3 API",
    },
    SensitivePort {
        port: 9090,
        protocol: Protocol::Tcp,
        risk: PortRisk::Medium,
        service: "Prometheus",
        description: "Prometheus metrics",
    },
    SensitivePort {
        port: 9100,
        protocol: Protocol::Tcp,
        risk: PortRisk::Medium,
        service: "Node Exporter",
        description: "Node metrics exporter",
    },
];

#[derive(Default)]
pub struct PortAnalyzer;

impl PortAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_ports(&self, services: &[Service]) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        for service in services {
            if service.ports.is_empty() {
                continue;
            }

            // Find management/database ports exposed to the world
            for port in &service.ports {
                if !port.exposed {
                    continue;
                }

                if let Some(sensitive) = SENSITIVE_PORTS
                    .iter()
                    .find(|s| s.port == port.port && s.protocol == port.protocol)
                {
                    let severity = match sensitive.risk {
                        PortRisk::Critical => Severity::Critical,
                        PortRisk::High => Severity::High,
                        PortRisk::Medium => Severity::Medium,
                        PortRisk::Low => Severity::Low,
                    };

                    findings.push(SecurityFinding {
                        id: Uuid::new_v4().to_string(),
                        severity,
                        title: format!("Exposed {} port", sensitive.service),
                        description: format!(
                            "Container '{}' exposes {} port {} ({}) to the host network",
                            service.name, sensitive.service, port.port, sensitive.description,
                        ),
                        source: "port_analysis".to_string(),
                        remediation: Some(format!(
                            "Bind port {} only to localhost (127.0.0.1) or remove the port mapping if not needed",
                            port.port,
                        )),
                        timestamp: SystemTime::now(),
                    });
                }
            }
        }

        findings
    }
}
