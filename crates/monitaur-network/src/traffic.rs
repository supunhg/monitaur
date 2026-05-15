use std::collections::HashMap;
use std::fs;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::Path;

use monitaur_core::error::{EngineError, EngineResult};
use monitaur_core::network::{Connection, ConnectionState};

/// Reads active TCP connections from /proc/net/tcp and /proc/net/tcp6.
pub fn read_active_connections() -> EngineResult<Vec<Connection>> {
    let mut connections = read_proc_net("/proc/net/tcp", false)?;
    connections.extend(read_proc_net("/proc/net/tcp6", true)?);
    Ok(connections)
}

fn read_proc_net(path: &str, is_v6: bool) -> EngineResult<Vec<Connection>> {
    let content = fs::read_to_string(Path::new(path))
        .map_err(|e| EngineError::Io(format!("Failed to read {path}: {e}")))?;

    let mut connections = Vec::new();

    for line in content.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 {
            continue;
        }

        // sl: local_address: rem_address: st: ...
        // parts[0] = sl, parts[1] = local_address, parts[2] = rem_address
        // parts[3] = st (state), parts[9] = inode

        let local = parse_addr_port(parts[1], is_v6)?;
        let remote = parse_addr_port(parts[2], is_v6)?;
        let state_num = u8::from_str_radix(parts[3], 16).unwrap_or(0);
        let state = tcp_state(state_num);
        let inode: u64 = parts[9].parse().unwrap_or(0);

        // Skip connections with 0.0.0.0:0 remote (listeners with no peer)
        if remote.1 == 0
            && (remote.0 == IpAddr::V4(Ipv4Addr::UNSPECIFIED)
                || remote.0 == IpAddr::V6(Ipv6Addr::UNSPECIFIED))
        {
            continue;
        }

        // Only include established connections for meaningful analysis
        if state != ConnectionState::Established {
            continue;
        }

        // Try to find owning PID via inode
        let pid = find_pid_for_inode(inode);

        connections.push(Connection {
            local_addr: local.0,
            local_port: local.1,
            remote_addr: remote.0,
            remote_port: remote.1,
            state,
            inode,
            pid,
            container_id: None,
        });
    }

    Ok(connections)
}

fn parse_addr_port(s: &str, is_v6: bool) -> EngineResult<(IpAddr, u16)> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err(EngineError::Network(format!(
            "Invalid addr:port format: {s}"
        )));
    }

    let ip_hex = parts[0];
    let port = u16::from_str_radix(parts[1], 16)
        .map_err(|e| EngineError::Network(format!("Invalid port hex: {e}")))?;

    let ip = if is_v6 {
        // /proc/net/tcp6: 32 hex chars, 8 groups of 4 (little-endian 128-bit)
        let groups: Vec<&str> = (0..8).map(|i| &ip_hex[i * 4..i * 4 + 4]).collect();

        // Convert from LE to BE (reverse pairs of groups)
        let be_groups: Vec<&str> = groups
            .chunks(2)
            .flat_map(|pair| pair.iter().rev())
            .copied()
            .collect();
        let addr_str = be_groups.join(":");

        // Pad with leading zeros
        let addr_str = addr_str
            .split(':')
            .map(|g| format!("{:0>4}", g))
            .collect::<Vec<_>>()
            .join(":");

        addr_str
            .parse::<Ipv6Addr>()
            .map(IpAddr::V6)
            .map_err(|e| EngineError::Network(format!("Invalid IPv6: {addr_str}: {e}")))?
    } else {
        // /proc/net/tcp: 8 hex chars, LE 32-bit
        let bytes = hex_to_u32(ip_hex).to_le_bytes();
        IpAddr::V4(Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]))
    };

    Ok((ip, port))
}

fn hex_to_u32(hex: &str) -> u32 {
    u32::from_str_radix(hex, 16).unwrap_or(0)
}

fn tcp_state(num: u8) -> ConnectionState {
    match num {
        0x01 => ConnectionState::Established,
        0x02 => ConnectionState::SynSent,
        0x03 => ConnectionState::SynReceived,
        0x04 => ConnectionState::FinWait1,
        0x05 => ConnectionState::FinWait2,
        0x06 => ConnectionState::TimeWait,
        0x07 => ConnectionState::Closed,
        0x08 => ConnectionState::CloseWait,
        0x09 => ConnectionState::LastAck,
        0x0A => ConnectionState::Listen,
        0x0B => ConnectionState::Closing,
        _ => ConnectionState::Closed,
    }
}

fn find_pid_for_inode(inode: u64) -> Option<u32> {
    let proc = Path::new("/proc");
    for entry in fs::read_dir(proc).ok()? {
        let entry = entry.ok()?;
        let pid: u32 = entry.file_name().to_string_lossy().parse().ok()?;

        let fd_dir = entry.path().join("fd");
        let fds = fs::read_dir(fd_dir).ok()?;
        for fd in fds {
            let fd = fd.ok()?;
            let link = fs::read_link(fd.path()).ok()?;
            let link_str = link.to_string_lossy();
            if link_str.contains(&format!("socket:[{inode}]")) {
                return Some(pid);
            }
        }
    }
    None
}

pub fn resolve_container_connections(
    connections: &mut [Connection],
    container_pids: &HashMap<String, Vec<u32>>,
) {
    for conn in connections.iter_mut() {
        if let Some(pid) = conn.pid {
            for (container_id, pids) in container_pids {
                if pids.contains(&pid) {
                    conn.container_id = Some(container_id.clone());
                    break;
                }
            }
        }
    }
}
