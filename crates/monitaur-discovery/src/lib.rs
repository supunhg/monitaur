pub mod dns;
pub mod docker;
pub mod network;
pub mod ports;

use monitaur_core::error::EngineResult;
use monitaur_core::models::InfraGraph;

#[derive(Default)]
pub struct DiscoveryEngine;

impl DiscoveryEngine {
    pub fn new() -> Self {
        Self
    }

    pub async fn discover(&self) -> EngineResult<InfraGraph> {
        todo!("implement full infrastructure discovery")
    }
}
