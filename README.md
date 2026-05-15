# Monitaur

Local-first infrastructure intelligence platform.

**Monitaur** helps you understand what's running on your infrastructure, what's exposed, how services communicate, and what can be hardened — all without sending data to the cloud.

## Architecture

Monitaur is composed of a Rust backend with modular engine crates and a future Tauri desktop frontend.

### Backend Engines

| Engine | Purpose |
|---|---|
| **Discovery** | Docker, network, port, and service discovery |
| **Monitoring** | Real-time metrics and container lifecycle |
| **Security** | Risk analysis, TLS checks, config auditing |
| **Network Intelligence** | Traffic analysis, DNS inspection, classification |
| **Visualization** | Topology and graph generation |
| **Metadata** | Caching, indexing, state snapshots |
| **Persistence** | SQLite-backed storage |

## Development

```bash
cargo build --workspace
cargo test --workspace
```

## License

MIT
