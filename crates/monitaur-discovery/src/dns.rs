use std::net::ToSocketAddrs;

use monitaur_core::error::EngineResult;

#[derive(Default)]
pub struct DnsResolver;

impl DnsResolver {
    pub fn new() -> Self {
        Self
    }

    pub fn resolve(&self, hostname: &str) -> EngineResult<Vec<String>> {
        let addrs: Vec<String> = (hostname, 0)
            .to_socket_addrs()
            .map_err(|e| {
                monitaur_core::error::EngineError::Discovery(format!(
                    "DNS resolution failed for {hostname}: {e}"
                ))
            })?
            .map(|addr| addr.ip().to_string())
            .collect();

        Ok(addrs)
    }
}
