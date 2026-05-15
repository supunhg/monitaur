pub mod health;
pub mod lifecycle;
pub mod metrics;

use std::time::Duration;

use monitaur_core::error::EngineResult;
use monitaur_core::metrics::MetricsSnapshot;
use monitaur_core::models::Service;
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::{info, warn};

pub struct MonitoringEngine {
    metrics: metrics::MetricsCollector,
    health: health::HealthChecker,
    poll_interval: Duration,
}

impl Default for MonitoringEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl MonitoringEngine {
    pub fn new() -> Self {
        Self {
            metrics: metrics::MetricsCollector::new(),
            health: health::HealthChecker::new(),
            poll_interval: Duration::from_secs(5),
        }
    }

    pub fn with_poll_interval(mut self, seconds: u64) -> Self {
        self.poll_interval = Duration::from_secs(seconds);
        self
    }

    pub async fn snapshot(&mut self, services: &[Service]) -> EngineResult<MetricsSnapshot> {
        let system = self.metrics.collect_system();

        let mut containers = Vec::with_capacity(services.len());
        for service in services {
            match self.metrics.collect_container(&service.id).await {
                Ok(cm) => containers.push(cm),
                Err(e) => {
                    warn!("Failed to collect metrics for {}: {e}", service.name);
                }
            }
        }

        let processes = self.health.check_services(services)?;

        Ok(MetricsSnapshot {
            system: Some(system),
            containers,
            processes,
            timestamp: std::time::SystemTime::now(),
        })
    }

    pub fn start_polling(mut self, services: Vec<Service>) -> UnboundedReceiver<MetricsSnapshot> {
        info!(
            "Starting monitoring poll every {}s",
            self.poll_interval.as_secs()
        );

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let interval = self.poll_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            loop {
                interval.tick().await;
                match self.snapshot(&services).await {
                    Ok(snapshot) => {
                        if tx.send(snapshot).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        warn!("Monitoring poll failed: {e}");
                    }
                }
            }
        });

        rx
    }

    pub fn poll_interval(&self) -> Duration {
        self.poll_interval
    }
}
