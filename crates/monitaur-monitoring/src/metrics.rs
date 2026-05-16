use std::time::SystemTime;

use bollard::Docker;
use bollard::container::StatsOptions;
use futures_util::StreamExt;
use monitaur_core::error::{EngineError, EngineResult};
use monitaur_core::metrics::ContainerMetrics;
use monitaur_core::metrics::SystemMetrics;
use sysinfo::{Networks, System};
use tracing::warn;

#[derive(Default)]
pub struct MetricsCollector {
    system: System,
    networks: Networks,
    docker: Option<Docker>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        let docker = Docker::connect_with_local_defaults().ok();
        if docker.is_none() {
            warn!("Docker socket not available — container metrics will be unavailable");
        }

        Self {
            system: System::new(),
            networks: Networks::new_with_refreshed_list(),
            docker,
        }
    }

    pub fn collect_system(&mut self) -> SystemMetrics {
        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        self.networks.refresh(true);

        let (rx, tx) = self.network_io();

        SystemMetrics {
            cpu_percent: self.system.global_cpu_usage() as f64,
            memory_total_bytes: self.system.total_memory(),
            memory_used_bytes: self.system.used_memory(),
            memory_percent: if self.system.total_memory() > 0 {
                (self.system.used_memory() as f64 / self.system.total_memory() as f64) * 100.0
            } else {
                0.0
            },
            network_rx_bytes: rx,
            network_tx_bytes: tx,
            timestamp: SystemTime::now(),
        }
    }

    pub async fn collect_container(&self, container_id: &str) -> EngineResult<ContainerMetrics> {
        let docker = self
            .docker
            .as_ref()
            .ok_or_else(|| EngineError::Monitoring("Docker not connected".to_string()))?;

        let stats = docker
            .stats(
                container_id,
                Some(StatsOptions {
                    one_shot: true,
                    ..Default::default()
                }),
            )
            .next()
            .await
            .ok_or_else(|| {
                EngineError::Monitoring(format!("No stats for container {container_id}"))
            })?
            .map_err(|e| EngineError::Monitoring(format!("Stats error for {container_id}: {e}")))?;

        let cpu_delta = stats.cpu_stats.cpu_usage.total_usage;
        let system_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0);
        let precpu_delta = stats.precpu_stats.cpu_usage.total_usage;
        let presystem_delta = stats.precpu_stats.system_cpu_usage.unwrap_or(0);
        let num_cpus = stats.cpu_stats.online_cpus.unwrap_or(1) as f64;

        let cpu_percent = if system_delta > 0 && presystem_delta > 0 {
            let cpu = (cpu_delta.saturating_sub(precpu_delta)) as f64
                / (system_delta.saturating_sub(presystem_delta)) as f64
                * num_cpus
                * 100.0;
            cpu.clamp(0.0, 100.0 * num_cpus)
        } else {
            0.0
        };

        let mem = stats.memory_stats;
        let memory_usage_bytes = mem.usage.unwrap_or(0);
        let memory_limit_bytes = mem.limit.unwrap_or(0);
        let memory_percent = if memory_limit_bytes > 0 {
            (memory_usage_bytes as f64 / memory_limit_bytes as f64) * 100.0
        } else {
            0.0
        };

        let (net_rx, net_tx) = stats
            .networks
            .as_ref()
            .map(|nets| {
                nets.values().fold((0u64, 0u64), |(rx, tx), net| {
                    (rx + net.rx_bytes, tx + net.tx_bytes)
                })
            })
            .unwrap_or((0, 0));

        Ok(ContainerMetrics {
            container_id: container_id.to_string(),
            cpu_percent,
            memory_usage_bytes,
            memory_limit_bytes,
            memory_percent,
            network_rx_bytes: net_rx,
            network_tx_bytes: net_tx,
            pids_current: Some(stats.num_procs as u64),
            timestamp: SystemTime::now(),
        })
    }

    fn network_io(&self) -> (u64, u64) {
        let mut rx = 0u64;
        let mut tx = 0u64;

        for (_name, data) in self.networks.iter() {
            rx = rx.saturating_add(data.total_received());
            tx = tx.saturating_add(data.total_transmitted());
        }

        (rx, tx)
    }
}
