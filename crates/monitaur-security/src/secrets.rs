use std::time::SystemTime;

use monitaur_core::models::{SecurityFinding, Service, Severity};
use regex::Regex;
use uuid::Uuid;

/// Patterns that look like secrets in environment variables or config values.
const SECRET_PATTERNS: &[(&str, Severity, &str)] = &[
    (
        r"(?i)(api[_-]?key|apikey)\s*[:=]\s*\S+",
        Severity::High,
        "API Key",
    ),
    (
        r"(?i)(secret|SECRET)\s*[:=]\s*\S{8,}",
        Severity::High,
        "Secret",
    ),
    (
        r"(?i)(password|PASSWORD|passwd)\s*[:=]\s*\S{4,}",
        Severity::Critical,
        "Password",
    ),
    (
        r"(?i)(token|TOKEN)\s*[:=]\s*\S{8,}",
        Severity::High,
        "Token",
    ),
    (
        r"(?i)(auth|AUTH)\s*[:=]\s*\S{8,}",
        Severity::High,
        "Auth Token",
    ),
    (r"(?i)(jwt|JWT)\s*[:=]\s*\S+", Severity::High, "JWT Token"),
    (
        r"(?i)-----BEGIN\s+(RSA\s+)?PRIVATE\s+KEY-----",
        Severity::Critical,
        "Private Key",
    ),
    (
        r"(?i)(connection[_-]?string|connstr)\s*[:=]\s*\S+",
        Severity::High,
        "Connection String",
    ),
    (
        r"(?i)(DATABASE_URL|REDIS_URL|MONGODB_URI|MYSQL_URL|PGHOST|PASSWORD)\s*[:=]\s*\S+",
        Severity::Critical,
        "Database URL",
    ),
];

#[derive(Default)]
pub struct SecretDetector;

impl SecretDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn detect_secrets(&self, services: &[Service]) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        let compiled: Vec<(Regex, Severity, String)> = SECRET_PATTERNS
            .iter()
            .filter_map(|(pattern, severity, label)| {
                Regex::new(pattern)
                    .ok()
                    .map(|re| (re, *severity, label.to_string()))
            })
            .collect();

        for service in services {
            for value in service.labels.values() {
                for (re, severity, label) in &compiled {
                    if re.is_match(value) {
                        let truncated = if value.len() > 60 {
                            format!("{}...", &value[..57])
                        } else {
                            value.clone()
                        };

                        findings.push(SecurityFinding {
                            id: Uuid::new_v4().to_string(),
                            severity: *severity,
                            title: format!("Potential {label} in configuration"),
                            description: format!(
                                "Container '{}' may have a {} in its configuration: \"{}\"",
                                service.name, label.to_lowercase(), truncated,
                            ),
                            source: "secret_detection".to_string(),
                            remediation: Some("Use Docker secrets or environment-specific secret injection rather than hardcoded values".to_string()),
                            timestamp: SystemTime::now(),
                        });
                    }
                }
            }
        }

        findings
    }
}
