pub mod config_audit;
pub mod port_analysis;
pub mod secrets;
pub mod tls;

#[derive(Default)]
pub struct SecurityEngine;

impl SecurityEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze(&self) {
        todo!("run all security analyzers")
    }
}
