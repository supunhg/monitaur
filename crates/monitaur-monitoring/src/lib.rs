pub mod health;
pub mod lifecycle;
pub mod metrics;

#[derive(Default)]
pub struct MonitoringEngine;

impl MonitoringEngine {
    pub fn new() -> Self {
        Self
    }

    pub async fn start(&self) {
        todo!("start metric collection and event subscriptions")
    }
}
