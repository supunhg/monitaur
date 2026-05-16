use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use monitaur_core::error::EngineResult;
use monitaur_core::metrics::MetricsSnapshot;
use monitaur_core::models::{InfraGraph, SecurityFinding};
use monitaur_core::network::NetworkAnalysis;
use rusqlite::Connection;
use tracing::info;

use crate::migrations;

fn unix_timestamp(t: SystemTime) -> i64 {
    t.duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    pub fn open(path: &str) -> EngineResult<Self> {
        let conn = Connection::open(path)
            .map_err(|e| monitaur_core::error::EngineError::Persistence(e.to_string()))?;

        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA temp_store=MEMORY;
             PRAGMA page_size=4096;
             PRAGMA cache_size=-64000;
             PRAGMA busy_timeout=5000;
             PRAGMA foreign_keys=ON;"
        )
        .map_err(|e| monitaur_core::error::EngineError::Persistence(e.to_string()))?;

        migrations::run_migrations(&conn)
            .map_err(|e| monitaur_core::error::EngineError::Persistence(e.to_string()))?;

        info!("SQLite store opened at {path}");
        Ok(Self { conn })
    }

    pub fn save_infra_graph(&self, graph: &InfraGraph) -> EngineResult<()> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| monitaur_core::error::EngineError::Persistence(e.to_string()))?;

        let now = unix_timestamp(SystemTime::now());

        for service in &graph.services {
            let labels_json =
                serde_json::to_string(&service.labels).unwrap_or_else(|_| "{}".to_string());

            tx.execute(
                "INSERT OR REPLACE INTO services (id, name, image, service_type, class, health, status, exposure_state, labels, discovered_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                rusqlite::params![
                    service.id,
                    service.name,
                    service.image,
                    format!("{:?}", service.service_type),
                    format!("{:?}", service.class),
                    format!("{:?}", service.health),
                    service.status,
                    format!("{:?}", service.exposure_state),
                    labels_json,
                    now,
                ],
            )
            .map_err(|e| monitaur_core::error::EngineError::Persistence(e.to_string()))?;

            for port in &service.ports {
                tx.execute(
                    "INSERT INTO ports (service_id, port, protocol, exposed) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![service.id, port.port, format!("{:?}", port.protocol), port.exposed as i32],
                )
                .map_err(|e| monitaur_core::error::EngineError::Persistence(e.to_string()))?;
            }

            for net in &service.networks {
                tx.execute(
                    "INSERT INTO service_networks (service_id, network_name) VALUES (?1, ?2)",
                    rusqlite::params![service.id, net],
                )
                .map_err(|e| monitaur_core::error::EngineError::Persistence(e.to_string()))?;
            }
        }

        for node in &graph.network_nodes {
            let addresses_json =
                serde_json::to_string(&node.addresses).unwrap_or_else(|_| "[]".to_string());
            tx.execute(
                "INSERT OR REPLACE INTO network_nodes (id, kind, addresses) VALUES (?1, ?2, ?3)",
                rusqlite::params![node.id, format!("{:?}", node.kind), addresses_json],
            )
            .map_err(|e| monitaur_core::error::EngineError::Persistence(e.to_string()))?;
        }

        for edge in &graph.edges {
            tx.execute(
                "INSERT INTO edges (source_id, target_id, relation, discovered_at) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![edge.source_id, edge.target_id, format!("{:?}", edge.relation), now],
            )
            .map_err(|e| monitaur_core::error::EngineError::Persistence(e.to_string()))?;
        }

        tx.commit()
            .map_err(|e| monitaur_core::error::EngineError::Persistence(e.to_string()))?;

        info!(
            "Saved {} services, {} nodes, {} edges",
            graph.services.len(),
            graph.network_nodes.len(),
            graph.edges.len()
        );
        Ok(())
    }

    pub fn save_metrics_snapshot(&self, snapshot: &MetricsSnapshot) -> EngineResult<i64> {
        let now = unix_timestamp(snapshot.timestamp);

        self.conn
            .execute(
                "INSERT INTO metrics_snapshots (cpu_percent, memory_total_bytes, memory_used_bytes, network_rx_bytes, network_tx_bytes, taken_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    snapshot.system.as_ref().map(|s| s.cpu_percent),
                    snapshot.system.as_ref().map(|s| s.memory_total_bytes as i64),
                    snapshot.system.as_ref().map(|s| s.memory_used_bytes as i64),
                    snapshot.system.as_ref().map(|s| s.network_rx_bytes as i64),
                    snapshot.system.as_ref().map(|s| s.network_tx_bytes as i64),
                    now,
                ],
            )
            .map_err(|e| monitaur_core::error::EngineError::Persistence(e.to_string()))?;

        let snapshot_id = self.conn.last_insert_rowid();

        for cm in &snapshot.containers {
            self.conn
                .execute(
                    "INSERT INTO container_metrics (snapshot_id, container_id, cpu_percent, memory_usage_bytes, memory_limit_bytes, network_rx_bytes, network_tx_bytes)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    rusqlite::params![
                        snapshot_id,
                        cm.container_id,
                        cm.cpu_percent,
                        cm.memory_usage_bytes as i64,
                        cm.memory_limit_bytes as i64,
                        cm.network_rx_bytes as i64,
                        cm.network_tx_bytes as i64,
                    ],
                )
                .map_err(|e| monitaur_core::error::EngineError::Persistence(e.to_string()))?;
        }

        info!(
            "Saved metrics snapshot #{snapshot_id} with {} containers",
            snapshot.containers.len()
        );
        Ok(snapshot_id)
    }

    pub fn save_network_analysis(&self, analysis: &NetworkAnalysis) -> EngineResult<()> {
        let now = unix_timestamp(SystemTime::now());

        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| monitaur_core::error::EngineError::Persistence(e.to_string()))?;

        for conn in &analysis.connections {
            tx.execute(
                "INSERT INTO connections (local_addr, local_port, remote_addr, remote_port, state, pid, container_id, detected_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    conn.local_addr.to_string(),
                    conn.local_port,
                    conn.remote_addr.to_string(),
                    conn.remote_port,
                    format!("{:?}", conn.state),
                    conn.pid,
                    conn.container_id,
                    now,
                ],
            )
            .map_err(|e| monitaur_core::error::EngineError::Persistence(e.to_string()))?;
        }

        for flow in &analysis.flows {
            tx.execute(
                "INSERT INTO traffic_flows (source, destination, port, class, connection_count, detected_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    flow.source,
                    flow.destination,
                    flow.port,
                    format!("{:?}", flow.class),
                    flow.connection_count as i64,
                    now,
                ],
            )
            .map_err(|e| monitaur_core::error::EngineError::Persistence(e.to_string()))?;
        }

        tx.commit()
            .map_err(|e| monitaur_core::error::EngineError::Persistence(e.to_string()))?;

        info!(
            "Saved {} connections, {} traffic flows",
            analysis.connections.len(),
            analysis.flows.len()
        );
        Ok(())
    }

    pub fn save_finding(&self, finding: &SecurityFinding) -> EngineResult<()> {
        let now = unix_timestamp(finding.timestamp);

        self.conn
            .execute(
                "INSERT INTO security_findings (id, severity, title, description, source, remediation, discovered_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    finding.id,
                    format!("{:?}", finding.severity),
                    finding.title,
                    finding.description,
                    finding.source,
                    finding.remediation,
                    now,
                ],
            )
            .map_err(|e| monitaur_core::error::EngineError::Persistence(e.to_string()))?;

        Ok(())
    }

    // ── Auth ──────────────────────────────────────────────────

    pub fn has_admin(&self) -> rusqlite::Result<bool> {
        self.conn
            .query_row("SELECT COUNT(*) FROM auth_config", [], |row| {
                row.get::<_, i64>(0)
            })
            .map(|count| count > 0)
    }

    pub fn set_password(&self, hash: &str) -> EngineResult<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO auth_config (id, password_hash) VALUES (1, ?1)",
                rusqlite::params![hash],
            )
            .map_err(|e| monitaur_core::error::EngineError::Persistence(e.to_string()))?;
        Ok(())
    }

    pub fn get_password_hash(&self) -> Option<String> {
        self.conn
            .query_row(
                "SELECT password_hash FROM auth_config WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .ok()
    }

    pub fn create_token(&self, token: &str) -> EngineResult<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        self.conn
            .execute(
                "INSERT INTO auth_tokens (token, created_at) VALUES (?1, ?2)",
                rusqlite::params![token, now],
            )
            .map_err(|e| monitaur_core::error::EngineError::Persistence(e.to_string()))?;
        Ok(())
    }

    pub fn validate_token(&self, token: &str) -> rusqlite::Result<bool> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM auth_tokens WHERE token = ?1 AND created_at > ?2",
                rusqlite::params![token, now - 604800], // 7 day TTL
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count > 0)
    }

    pub fn cleanup_expired_tokens(&self) -> EngineResult<usize> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let count = self
            .conn
            .execute(
                "DELETE FROM auth_tokens WHERE created_at < ?1",
                rusqlite::params![now - 604800],
            )
            .map_err(|e| monitaur_core::error::EngineError::Persistence(e.to_string()))?;
        if count > 0 {
            info!("Cleaned up {count} expired auth tokens");
        }
        Ok(count)
    }

    // ── Historical reads ────────────────────────────────────────

    pub fn list_metrics_history(&self, limit: usize) -> EngineResult<Vec<monitaur_core::metrics::MetricsSnapshot>> {
        // Single JOIN query eliminates N+1 problem
        let mut stmt = self
            .conn
            .prepare(
                "SELECT ms.id, ms.cpu_percent, ms.memory_total_bytes, ms.memory_used_bytes,
                        ms.network_rx_bytes, ms.network_tx_bytes, ms.taken_at,
                        cm.container_id, cm.cpu_percent, cm.memory_usage_bytes,
                        cm.memory_limit_bytes, cm.network_rx_bytes, cm.network_tx_bytes
                 FROM metrics_snapshots ms
                 LEFT JOIN container_metrics cm ON cm.snapshot_id = ms.id
                 ORDER BY ms.taken_at DESC
                 LIMIT ?1",
            )
            .map_err(|e| monitaur_core::error::EngineError::Persistence(e.to_string()))?;

        let rows = stmt
            .query_map(rusqlite::params![limit as i64], |row| {
                let taken: i64 = row.get(6)?;
                let ts = std::time::UNIX_EPOCH + std::time::Duration::from_secs(taken as u64);

                let container_id: Option<String> = row.get(7)?;
                let cm_cpu: f64 = row.get::<_, Option<f64>>(8)?.unwrap_or(0.0);
                let cm_mem_usage: i64 = row.get::<_, Option<i64>>(9)?.unwrap_or(0);
                let cm_mem_limit: i64 = row.get::<_, Option<i64>>(10)?.unwrap_or(0);
                let cm_net_rx: i64 = row.get::<_, Option<i64>>(11)?.unwrap_or(0);
                let cm_net_tx: i64 = row.get::<_, Option<i64>>(12)?.unwrap_or(0);

                let cm = container_id.map(|cid| {
                    let mem_usage = cm_mem_usage as u64;
                    let mem_limit = cm_mem_limit as u64;
                    let mem_pct = if mem_limit > 0 { (mem_usage as f64 / mem_limit as f64) * 100.0 } else { 0.0 };
                    monitaur_core::metrics::ContainerMetrics {
                        container_id: cid,
                        cpu_percent: cm_cpu,
                        memory_usage_bytes: mem_usage,
                        memory_limit_bytes: mem_limit,
                        memory_percent: mem_pct,
                        network_rx_bytes: cm_net_rx as u64,
                        network_tx_bytes: cm_net_tx as u64,
                        pids_current: None,
                        timestamp: ts,
                    }
                });

                let snapshot_id: i64 = row.get(0)?;
                let cpu: Option<f64> = row.get(1)?;
                let mem_total: Option<i64> = row.get(2)?;
                let mem_used: Option<i64> = row.get(3)?;
                let rx: Option<i64> = row.get(4)?;
                let tx: Option<i64> = row.get(5)?;

                Ok((snapshot_id, ts, cpu, mem_total, mem_used, rx, tx, cm))
            })
            .map_err(|e| monitaur_core::error::EngineError::Persistence(e.to_string()))?;

        // Group by snapshot_id using a HashMap to avoid O(n*m)
        let mut snapshot_map: std::collections::HashMap<i64, monitaur_core::metrics::MetricsSnapshot> = std::collections::HashMap::new();
        let mut order: Vec<i64> = Vec::new();

        for row in rows {
            let (id, ts, cpu, mem_total, mem_used, rx, tx, cm) =
                row.map_err(|e| monitaur_core::error::EngineError::Persistence(e.to_string()))?;

            let entry = snapshot_map.entry(id).or_insert_with(|| {
                order.push(id);
                monitaur_core::metrics::MetricsSnapshot {
                    system: cpu.map(|_| monitaur_core::metrics::SystemMetrics {
                        cpu_percent: cpu.unwrap_or(0.0),
                        memory_total_bytes: mem_total.unwrap_or(0) as u64,
                        memory_used_bytes: mem_used.unwrap_or(0) as u64,
                        memory_percent: mem_total.filter(|&t| t > 0).map(|t| {
                            (mem_used.unwrap_or(0) as f64 / t as f64) * 100.0
                        }).unwrap_or(0.0),
                        network_rx_bytes: rx.unwrap_or(0) as u64,
                        network_tx_bytes: tx.unwrap_or(0) as u64,
                        timestamp: ts,
                    }),
                    containers: Vec::new(),
                    processes: Vec::new(),
                    timestamp: ts,
                }
            });

            if let Some(cm_val) = cm {
                entry.containers.push(cm_val);
            }
        }

        // Return in order
        let snapshots: Vec<_> = order.iter().filter_map(|id| snapshot_map.remove(id)).collect();
        Ok(snapshots)
    }

    pub fn list_findings(
        &self,
        limit: usize,
        severity_filter: Option<&str>,
    ) -> EngineResult<Vec<SecurityFinding>> {
        let (query, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match severity_filter {
            Some(sev) => (
                "SELECT id, severity, title, description, source, remediation, discovered_at
                 FROM security_findings WHERE severity = ?1 ORDER BY discovered_at DESC LIMIT ?2",
                vec![Box::new(sev.to_string()) as Box<dyn rusqlite::types::ToSql>, Box::new(limit as i64)],
            ),
            None => (
                "SELECT id, severity, title, description, source, remediation, discovered_at
                 FROM security_findings ORDER BY discovered_at DESC LIMIT ?1",
                vec![Box::new(limit as i64) as Box<dyn rusqlite::types::ToSql>],
            ),
        };

        let mut stmt = self
            .conn
            .prepare(query)
            .map_err(|e| monitaur_core::error::EngineError::Persistence(e.to_string()))?;

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let rows = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(SecurityFinding {
                    id: row.get(0)?,
                    severity: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(1)?))
                        .unwrap_or(monitaur_core::models::Severity::Info),
                    title: row.get(2)?,
                    description: row.get(3)?,
                    source: row.get(4)?,
                    remediation: row.get(5)?,
                    timestamp: std::time::UNIX_EPOCH
                        + std::time::Duration::from_secs(row.get::<_, i64>(6)? as u64),
                })
            })
            .map_err(|e| monitaur_core::error::EngineError::Persistence(e.to_string()))?;

        let mut findings = Vec::new();
        for row in rows {
            findings.push(
                row.map_err(|e| monitaur_core::error::EngineError::Persistence(e.to_string()))?,
            );
        }
        Ok(findings)
    }
}
