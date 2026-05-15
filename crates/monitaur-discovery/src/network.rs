// System interfaces and local network scanning

#[derive(Default)]
pub struct NetworkDiscoverer;

impl NetworkDiscoverer {
    pub fn new() -> Self {
        Self
    }

    pub fn scan_interfaces(&self) {
        todo!("enumerate system network interfaces")
    }
}
