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

        conn.execute_batch("PRAGMA journal_mode=WAL;")
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
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM auth_tokens WHERE token = ?1",
                rusqlite::params![token],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count > 0)
    }
}
