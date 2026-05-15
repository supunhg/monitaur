use rusqlite::Connection;
use tracing::info;

const SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS services (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    image TEXT,
    service_type TEXT NOT NULL,
    class TEXT NOT NULL,
    health TEXT NOT NULL,
    status TEXT NOT NULL,
    exposure_state TEXT NOT NULL,
    labels TEXT NOT NULL DEFAULT '{}',
    discovered_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS ports (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    service_id TEXT NOT NULL REFERENCES services(id),
    port INTEGER NOT NULL,
    protocol TEXT NOT NULL,
    exposed INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS service_networks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    service_id TEXT NOT NULL REFERENCES services(id),
    network_name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS network_nodes (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    addresses TEXT NOT NULL DEFAULT '[]'
);

CREATE TABLE IF NOT EXISTS edges (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    relation TEXT NOT NULL,
    discovered_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS metrics_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cpu_percent REAL,
    memory_total_bytes INTEGER,
    memory_used_bytes INTEGER,
    network_rx_bytes INTEGER,
    network_tx_bytes INTEGER,
    taken_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS container_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    snapshot_id INTEGER NOT NULL REFERENCES metrics_snapshots(id),
    container_id TEXT NOT NULL,
    cpu_percent REAL,
    memory_usage_bytes INTEGER,
    memory_limit_bytes INTEGER,
    network_rx_bytes INTEGER,
    network_tx_bytes INTEGER
);

CREATE TABLE IF NOT EXISTS security_findings (
    id TEXT PRIMARY KEY,
    severity TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    source TEXT NOT NULL,
    remediation TEXT,
    service_id TEXT REFERENCES services(id),
    discovered_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS connections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    local_addr TEXT NOT NULL,
    local_port INTEGER NOT NULL,
    remote_addr TEXT NOT NULL,
    remote_port INTEGER NOT NULL,
    state TEXT NOT NULL,
    pid INTEGER,
    container_id TEXT,
    detected_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS traffic_flows (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source TEXT NOT NULL,
    destination TEXT NOT NULL,
    port INTEGER NOT NULL,
    class TEXT NOT NULL,
    connection_count INTEGER NOT NULL,
    detected_at INTEGER NOT NULL
);
";

pub fn run_migrations(conn: &Connection) -> rusqlite::Result<()> {
    let version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if version < 1 {
        info!("Running initial schema migration");
        conn.execute_batch(SCHEMA_SQL)?;
        conn.execute("INSERT INTO schema_version (version) VALUES (1)", [])?;
    }

    info!("Database schema at version 1");
    Ok(())
}
