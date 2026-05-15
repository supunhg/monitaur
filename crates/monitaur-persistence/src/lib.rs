pub mod migrations;
pub mod sqlite;

#[derive(Default)]
pub struct PersistenceEngine;

impl PersistenceEngine {
    pub fn new(_path: &str) -> Self {
        Self
    }

    pub fn store(&self) {
        todo!("store state to persistent storage")
    }
}
