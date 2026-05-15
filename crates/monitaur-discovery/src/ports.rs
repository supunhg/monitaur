// Port scanning and service detection

#[derive(Default)]
pub struct PortScanner;

impl PortScanner {
    pub fn new() -> Self {
        Self
    }

    pub fn scan(&self, _targets: &[String]) {
        todo!("scan ports on target addresses")
    }
}
