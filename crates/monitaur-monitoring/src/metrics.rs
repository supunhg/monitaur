// CPU, memory, network throughput collection

#[derive(Default)]
pub struct MetricsCollector;

impl MetricsCollector {
    pub fn new() -> Self {
        Self
    }

    pub fn collect(&self) {
        todo!("collect system metrics")
    }
}
