use std::time::SystemTime;

use monitaur_core::error::EngineResult;
use monitaur_core::metrics::ProcessHealth;
use monitaur_core::models::Service;

#[derive(Default)]
pub struct HealthChecker;

impl HealthChecker {
    pub fn new() -> Self {
        Self
    }

    pub fn check_services(&self, services: &[Service]) -> EngineResult<Vec<ProcessHealth>> {
        let results: Vec<ProcessHealth> = services
            .iter()
            .map(|s| ProcessHealth {
                service_id: s.id.clone(),
                status: s.status.clone(),
                uptime_seconds: None,
                last_check: SystemTime::now(),
            })
            .collect();

        Ok(results)
    }
}
