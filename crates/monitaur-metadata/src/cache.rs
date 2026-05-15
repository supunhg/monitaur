// In-memory caching for entity data

#[derive(Default)]
pub struct EntityCache;

impl EntityCache {
    pub fn new() -> Self {
        Self
    }

    pub fn get(&self) {
        todo!("retrieve cached entity")
    }
}
