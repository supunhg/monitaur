use std::collections::HashMap;
use std::fs;
use std::net::IpAddr;
use std::path::Path;

use monitaur_core::error::{EngineError, EngineResult};
use monitaur_core::network::DnsQuery;

/// Reads the system DNS resolver configuration from /etc/resolv.conf.
pub fn read_resolv_conf() -> EngineResult<Vec<IpAddr>> {
    let content = fs::read_to_string(Path::new("/etc/resolv.conf"))
        .map_err(|e| EngineError::Io(format!("Failed to read /etc/resolv.conf: {e}")))?;

    let mut servers = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if let Some(addr) = line.strip_prefix("nameserver ")
            && let Ok(ip) = addr.parse::<IpAddr>()
        {
            servers.push(ip);
        }
    }
    Ok(servers)
}

/// Checks /etc/hosts for known hostname mappings.
pub fn read_etc_hosts() -> EngineResult<HashMap<String, Vec<IpAddr>>> {
    let content = fs::read_to_string(Path::new("/etc/hosts"))
        .map_err(|e| EngineError::Io(format!("Failed to read /etc/hosts: {e}")))?;

    let mut mappings: HashMap<String, Vec<IpAddr>> = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        let ip: IpAddr = match parts[0].parse() {
            Ok(ip) => ip,
            Err(_) => continue,
        };
        for hostname in &parts[1..] {
            mappings.entry(hostname.to_string()).or_default().push(ip);
        }
    }
    Ok(mappings)
}

/// Resolves known DNS names from /etc/hosts (no live DNS queries).
pub fn resolve_known_hosts() -> EngineResult<Vec<DnsQuery>> {
    let hosts = read_etc_hosts()?;
    Ok(hosts
        .into_iter()
        .map(|(query, response)| DnsQuery {
            query,
            query_type: "A".to_string(),
            response,
        })
        .collect())
}
