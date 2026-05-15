use crate::models::Port;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineEvent {
    ContainerStarted { id: String, name: String },
    ContainerStopped { id: String },
    ServiceExposed { service_id: String, port: Port },
    SuspiciousTrafficDetected { source: String, destination: String },
    TlsCertificateExpired { service_id: String },
}
