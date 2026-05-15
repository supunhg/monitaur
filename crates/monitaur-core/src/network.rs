use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ConnectionState {
    Established,
    SynSent,
    SynReceived,
    FinWait1,
    FinWait2,
    TimeWait,
    Closed,
    CloseWait,
    LastAck,
    Listen,
    Closing,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Connection {
    pub local_addr: IpAddr,
    pub local_port: u16,
    pub remote_addr: IpAddr,
    pub remote_port: u16,
    pub state: ConnectionState,
    pub inode: u64,
    pub pid: Option<u32>,
    pub container_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrafficClass {
    Http,
    Https,
    Database,
    Cache,
    Dns,
    Ssh,
    Smtp,
    MessageQueue,
    ContainerOrchestration,
    Monitoring,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrafficFlow {
    pub source: String,
    pub destination: String,
    pub port: u16,
    pub class: TrafficClass,
    pub connection_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DnsQuery {
    pub query: String,
    pub query_type: String,
    pub response: Vec<IpAddr>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkAnalysis {
    pub connections: Vec<Connection>,
    pub flows: Vec<TrafficFlow>,
    pub dns_queries: Vec<DnsQuery>,
}
