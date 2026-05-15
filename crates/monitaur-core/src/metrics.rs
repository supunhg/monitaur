use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemMetrics {
    pub cpu_percent: f64,
    pub memory_total_bytes: u64,
    pub memory_used_bytes: u64,
    pub memory_percent: f64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContainerMetrics {
    pub container_id: String,
    pub cpu_percent: f64,
    pub memory_usage_bytes: u64,
    pub memory_limit_bytes: u64,
    pub memory_percent: f64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub pids_current: Option<u64>,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessHealth {
    pub service_id: String,
    pub status: String,
    pub uptime_seconds: Option<u64>,
    pub last_check: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LifecycleEvent {
    Started {
        container_id: String,
        name: String,
    },
    Stopped {
        container_id: String,
    },
    Died {
        container_id: String,
        exit_code: i64,
    },
    HealthStatus {
        container_id: String,
        status: String,
    },
    Paused {
        container_id: String,
    },
    Unpaused {
        container_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetricsSnapshot {
    pub system: Option<SystemMetrics>,
    pub containers: Vec<ContainerMetrics>,
    pub processes: Vec<ProcessHealth>,
    pub timestamp: SystemTime,
}
