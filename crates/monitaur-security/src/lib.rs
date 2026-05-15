pub mod config_audit;
pub mod port_analysis;
pub mod secrets;
pub mod tls;

use monitaur_core::models::{SecurityFinding, Service};
use tracing::info;

#[derive(Default)]
pub struct SecurityEngine;

impl SecurityEngine {
    pub fn new() -> Self {
        Self
    }

    pub async fn analyze(&self, services: &[Service]) -> Vec<SecurityFinding> {
        info!("Running security analysis on {} services", services.len());

        let port_analyzer = port_analysis::PortAnalyzer::new();
        let auditor = config_audit::ConfigAuditor::new();
        let secret_detector = secrets::SecretDetector::new();
        let tls_checker = tls::TlsChecker::new();

        let mut findings = Vec::new();

        // Port analysis
        let port_findings = port_analyzer.analyze_ports(services);
        findings.extend(port_findings);

        // Config audit
        let config_findings = auditor.audit_services(services);
        findings.extend(config_findings);

        // Secret detection
        let secret_findings = secret_detector.detect_secrets(services);
        findings.extend(secret_findings);

        // TLS checks
        let tls_findings = tls_checker.check_service_tls(services).await;
        findings.extend(tls_findings);

        info!("Security analysis complete: {} findings", findings.len());
        findings
    }
}
