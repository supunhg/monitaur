pub mod cache;
pub mod indexing;
pub mod snapshots;

#[derive(Default)]
pub struct MetadataEngine;

impl MetadataEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn index(&self) {
        todo!("maintain normalized system state")
    }
}
