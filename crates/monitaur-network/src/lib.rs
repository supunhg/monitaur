pub const NAME: &str = "monitaur-network";

pub mod classification;
pub mod dns_inspection;
pub mod traffic;

#[derive(Default)]
pub struct NetworkIntelligenceEngine;

impl NetworkIntelligenceEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze(&self) {
        todo!("analyze network traffic and connections")
    }
}
