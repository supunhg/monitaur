use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServiceType {
    Container,
    Process,
    Application,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Protocol {
    Tcp,
    Udp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Port {
    pub port: u16,
    pub protocol: Protocol,
    pub exposed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Health {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExposureState {
    Exposed,
    Internal,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServiceClass {
    Database,
    Cache,
    ReverseProxy,
    WebApp,
    Worker,
    Messaging,
    Monitoring,
    Security,
    Utility,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Service {
    pub id: String,
    pub name: String,
    pub image: Option<String>,
    pub service_type: ServiceType,
    pub class: ServiceClass,
    pub ports: Vec<Port>,
    pub networks: Vec<String>,
    pub health: Health,
    pub status: String,
    pub labels: HashMap<String, String>,
    pub exposure_state: ExposureState,
}

// ---------------------------------------------------------------------------
// Network Node
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetworkNodeKind {
    InternalService,
    ExternalService,
    Domain,
    Endpoint,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkNode {
    pub id: String,
    pub kind: NetworkNodeKind,
    pub addresses: Vec<String>,
}

// ---------------------------------------------------------------------------
// Security Finding
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SecurityFinding {
    pub id: String,
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub source: String,
    pub remediation: Option<String>,
    pub timestamp: SystemTime,
}

// ---------------------------------------------------------------------------
// Graph
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelationType {
    DependsOn,
    Exposes,
    ConnectsTo,
    Contains,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Edge {
    pub source_id: String,
    pub target_id: String,
    pub relation: RelationType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InfraGraph {
    pub services: Vec<Service>,
    pub network_nodes: Vec<NetworkNode>,
    pub edges: Vec<Edge>,
}
