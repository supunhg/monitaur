// SQLite database connection and operations

#[derive(Default)]
pub struct SqliteStore;

impl SqliteStore {
    pub fn new(_path: &str) -> Self {
        Self
    }

    pub fn query(&self) {
        todo!("execute SQLite queries")
    }
}
