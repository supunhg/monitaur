use std::net::SocketAddr;
use std::time::{Duration, SystemTime};

use monitaur_core::models::{SecurityFinding, Service, Severity};
use tokio::net::TcpStream;
use uuid::Uuid;

#[derive(Default)]
pub struct TlsChecker;

impl TlsChecker {
    pub fn new() -> Self {
        Self
    }

    /// Checks if TLS-enabled ports (443, 8443, 6443) accept connections.
    pub async fn check_service_tls(&self, services: &[Service]) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        for service in services {
            for port in &service.ports {
                if port.port != 443 && port.port != 8443 && port.port != 6443 {
                    continue;
                }
                if !port.exposed {
                    continue;
                }

                let addr: SocketAddr = match format!("127.0.0.1:{}", port.port).parse() {
                    Ok(a) => a,
                    Err(_) => continue,
                };

                match tokio::time::timeout(Duration::from_secs(2), TcpStream::connect(addr)).await {
                    Ok(Ok(_)) => {
                        // Port is open and accepting connections
                        // Full TLS inspection (cert expiry, ciphers) is a future enhancement
                        findings.push(SecurityFinding {
                            id: Uuid::new_v4().to_string(),
                            severity: Severity::Info,
                            title: "TLS port accepting connections".to_string(),
                            description: format!(
                                "Service '{}' has port {} open — TLS certificate inspection is not yet implemented",
                                service.name, port.port,
                            ),
                            source: "tls_check".to_string(),
                            remediation: None,
                            timestamp: SystemTime::now(),
                        });
                    }
                    _ => {
                        findings.push(SecurityFinding {
                            id: Uuid::new_v4().to_string(),
                            severity: Severity::Info,
                            title: "Non-TLS connection on secure port".to_string(),
                            description: format!(
                                "Service '{}' on port {} does not accept connections (port may be closed or filtered)",
                                service.name, port.port,
                            ),
                            source: "tls_check".to_string(),
                            remediation: Some("Enable TLS on this port for secure communication".to_string()),
                            timestamp: SystemTime::now(),
                        });
                    }
                }
            }
        }

        findings
    }
}
