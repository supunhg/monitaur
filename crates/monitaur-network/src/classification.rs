use std::collections::HashMap;

use monitaur_core::network::{Connection, TrafficClass, TrafficFlow};

/// Classifies connections into traffic flows grouped by destination.
pub fn build_traffic_flows(connections: &[Connection]) -> Vec<TrafficFlow> {
    let mut flow_map: HashMap<(String, u16), (TrafficClass, usize)> = HashMap::new();

    for conn in connections {
        let class = classify_port(conn.remote_port);
        let dest = conn.remote_addr.to_string();
        let entry = flow_map
            .entry((dest, conn.remote_port))
            .or_insert((class, 0));
        entry.1 += 1;
    }

    flow_map
        .into_iter()
        .map(|((dest, port), (class, count))| TrafficFlow {
            source: "localhost".to_string(),
            destination: dest,
            port,
            class,
            connection_count: count,
        })
        .collect()
}

/// Classify traffic by destination port.
fn classify_port(port: u16) -> TrafficClass {
    match port {
        80 => TrafficClass::Http,
        443 | 8443 => TrafficClass::Https,
        22 => TrafficClass::Ssh,
        25 | 465 | 587 => TrafficClass::Smtp,
        53 => TrafficClass::Dns,
        3306 | 5432 | 5433 | 5434 | 1521 | 1433 | 1186 | 3320 => TrafficClass::Database,
        6379 | 11211 => TrafficClass::Cache,
        5672 | 5671 | 61613 | 61614 | 1883 | 8883 => TrafficClass::MessageQueue,
        2375 | 2376 | 6443 => TrafficClass::ContainerOrchestration,
        9090 | 9092 | 9100 | 9200 | 9419 | 16686 | 14250 | 4317 => TrafficClass::Monitoring,
        _ => TrafficClass::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn make_conn(remote_port: u16) -> Connection {
        Connection {
            local_addr: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            local_port: 40000,
            remote_addr: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            remote_port,
            state: monitaur_core::network::ConnectionState::Established,
            inode: 0,
            pid: None,
            container_id: None,
        }
    }

    #[test]
    fn test_classify_http() {
        assert_eq!(classify_port(80), TrafficClass::Http);
    }

    #[test]
    fn test_classify_https() {
        assert_eq!(classify_port(443), TrafficClass::Https);
    }

    #[test]
    fn test_classify_database() {
        assert_eq!(classify_port(5432), TrafficClass::Database);
        assert_eq!(classify_port(3306), TrafficClass::Database);
        assert_eq!(classify_port(6379), TrafficClass::Cache);
    }

    #[test]
    fn test_classify_unknown() {
        assert_eq!(classify_port(9999), TrafficClass::Unknown);
    }

    #[test]
    fn test_build_flows_groups_by_destination() {
        let conns = vec![make_conn(443), make_conn(443), make_conn(80)];
        let flows = build_traffic_flows(&conns);
        assert_eq!(flows.len(), 2);
        let https = flows.iter().find(|f| f.port == 443).unwrap();
        assert_eq!(https.connection_count, 2);
    }
}
