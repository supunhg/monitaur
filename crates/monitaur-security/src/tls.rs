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
                    Ok(Ok(_)) => {}
                    _ => {
                        findings.push(SecurityFinding {
                            id: Uuid::new_v4().to_string(),
                            severity: Severity::Low,
                            title: "Secure port unreachable".to_string(),
                            description: format!(
                                "Service '{}' exposes secure port {} but it did not accept a TCP connection from the host",
                                service.name, port.port,
                            ),
                            source: "tls_check".to_string(),
                            remediation: Some("Verify the service is listening on the exposed secure port and that host-to-container networking is configured correctly".to_string()),
                            timestamp: SystemTime::now(),
                        });
                    }
                }
            }
        }

        findings
    }
}
