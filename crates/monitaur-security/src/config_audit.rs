use std::time::SystemTime;

use monitaur_core::models::{SecurityFinding, Service, ServiceClass, Severity};
use uuid::Uuid;

#[derive(Default)]
pub struct ConfigAuditor;

impl ConfigAuditor {
    pub fn new() -> Self {
        Self
    }

    pub fn audit_services(&self, services: &[Service]) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        for service in services {
            // Check if running as root
            if let Some(root) = service.labels.get("root")
                && root == "true"
            {
                findings.push(SecurityFinding {
                    id: Uuid::new_v4().to_string(),
                    severity: Severity::High,
                    title: "Container running as root".to_string(),
                    description: format!(
                        "Container '{}' is running as root. This is a security risk if the container is compromised.",
                        service.name,
                    ),
                    source: "config_audit".to_string(),
                    remediation: Some("Use a non-root user in the Dockerfile with USER directive".to_string()),
                    timestamp: SystemTime::now(),
                });
            }

            // Check for privileged mode
            if let Some(privileged) = service.labels.get("privileged")
                && privileged == "true"
            {
                findings.push(SecurityFinding {
                    id: Uuid::new_v4().to_string(),
                    severity: Severity::Critical,
                    title: "Container running in privileged mode".to_string(),
                    description: format!(
                        "Container '{}' is running in privileged mode. This grants full host access to the container.",
                        service.name,
                    ),
                    source: "config_audit".to_string(),
                    remediation: Some("Remove --privileged flag and use fine-grained capabilities instead".to_string()),
                    timestamp: SystemTime::now(),
                });
            }

            // Check for host network mode
            if let Some(network_mode) = service.labels.get("network_mode")
                && network_mode == "host"
            {
                findings.push(SecurityFinding {
                    id: Uuid::new_v4().to_string(),
                    severity: Severity::High,
                    title: "Container using host network".to_string(),
                    description: format!(
                        "Container '{}' uses host network mode, bypassing Docker network isolation.",
                        service.name,
                    ),
                    source: "config_audit".to_string(),
                    remediation: Some("Use Docker bridge networks instead of --net=host".to_string()),
                    timestamp: SystemTime::now(),
                });
            }

            // Check for database or cache exposed publicly
            if (service.class == ServiceClass::Database || service.class == ServiceClass::Cache)
                && service.exposure_state == monitaur_core::models::ExposureState::Exposed
            {
                findings.push(SecurityFinding {
                    id: Uuid::new_v4().to_string(),
                    severity: Severity::High,
                    title: "Database exposed to host network".to_string(),
                    description: format!(
                        "Data store '{}' ({:?}) is exposed on the host network. This can allow unauthorized access.",
                        service.name, service.class,
                    ),
                    source: "config_audit".to_string(),
                    remediation: Some("Bind to localhost only (127.0.0.1) or use an internal Docker network".to_string()),
                    timestamp: SystemTime::now(),
                });
            }

            // Check for services running on well-known ports without TLS
            if service.class == ServiceClass::WebApp
                && service.exposure_state == monitaur_core::models::ExposureState::Exposed
            {
                let has_tls = service
                    .ports
                    .iter()
                    .any(|p| p.port == 443 || p.port == 8443);
                let has_http = service
                    .ports
                    .iter()
                    .any(|p| p.port == 80 || p.port == 8080 || p.port == 3000 || p.port == 4000);

                if has_http && !has_tls {
                    findings.push(SecurityFinding {
                        id: Uuid::new_v4().to_string(),
                        severity: Severity::Medium,
                        title: "Web service without TLS".to_string(),
                        description: format!(
                            "Service '{}' has HTTP exposed but no HTTPS port detected.",
                            service.name,
                        ),
                        source: "config_audit".to_string(),
                        remediation: Some("Enable TLS and redirect HTTP to HTTPS".to_string()),
                        timestamp: SystemTime::now(),
                    });
                }
            }
        }

        findings
    }
}
