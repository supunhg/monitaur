// DNS resolution and reverse DNS lookups

#[derive(Default)]
pub struct DnsResolver;

impl DnsResolver {
    pub fn new() -> Self {
        Self
    }

    pub fn resolve(&self, _hostname: &str) {
        todo!("resolve hostname to IP addresses")
    }
}
