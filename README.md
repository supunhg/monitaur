# monitaur

**local-first infrastructure intelligence**

monitaur helps you understand what's running on your machine, what's exposed to the network, how services communicate, and where security risks lie — all without sending data to the cloud.

---

## architecture

```
┌─────────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐
│  dashboard  │  │ topology │  │ security │  │ services │
│  (live cpus │  │ (cyto-   │  │ (findings│  │ (inspector│
│   mem net)  │  │  scope)  │  │  filter) │  │  detail)  │
└──────┬──────┘  └────┬─────┘  └────┬─────┘  └─────┬────┘
       └──────────────┴──────────────┴──────────────┘
                        │
                   react + vite + tailwind
                        │ http proxy (vite dev) / tauri (desktop)
              ┌─────────▼─────────┐
              │   axum rest api   │
              │  localhost:8080   │
              └─────────┬─────────┘
          ┌─────────────┼─────────────┐
          │             │             │
    ┌─────▼─────┐ ┌────▼────┐ ┌──────▼──────┐
    │ discovery │ │ monitor │ │  security   │
    │ engine    │ │ engine  │ │  engine     │
    └───────────┘ └─────────┘ └─────────────┘
    ┌─────▼─────┐ ┌────▼────┐ ┌──────▼──────┐
    │ network   │ │metadata │ │visualization │
    │ engine    │ │ engine  │ │  engine      │
    └───────────┘ └─────────┘ └─────────────┘
          │
    ┌─────▼─────┐
    │  sqlite   │
    └───────────┘
```

## what it does

- **docker discovery** — enumerates containers, images, ports, networks
- **live metrics** — cpu, memory, network i/o (system + per-container)
- **security analysis** — exposed port risk, container config audits, secret scanning, tls checks
- **network intelligence** — reads `/proc/net/tcp` to map active outbound connections, classifies traffic by port (https, dns, database, etc.)
- **topology visualization** — interactive cytoscape.js graph with layers, clustering, hover/click inspection
- **persistence** — all results stored in sqlite for historical queries

## quick start

```bash
# terminal 1: start the api server
cargo run -- serve --port 8080

# terminal 2: start the frontend dev server
pnpm dev

# or run a one-shot scan
cargo run -- scan
```

open http://localhost:5173

## project layout

```
monitaur/
├── src/                  # cli + api server
│   ├── main.rs           # clap: scan | serve
│   ├── api.rs            # 8 rest endpoints
│   └── app_state.rs      # shared engine state
├── crates/
│   ├── core/             # types, models, events
│   ├── discovery/        # docker containers, ports, networks
│   ├── monitoring/       # cpu, memory, container stats
│   ├── security/         # port risk, config audit, secrets, tls
│   ├── network/          # /proc/net/tcp, traffic classification
│   ├── visualization/    # topology, clustering, graph optimization
│   ├── metadata/         # caching, indexing, snapshots
│   └── persistence/      # sqlite store
├── apps/frontend/        # react + vite + tailwind
│   ├── src/pages/        # dashboard, topology, security, services
│   ├── src/components/   # shell, cytoscape graph, error boundary
│   └── src/hooks/        # react query + live polling
└── src-tauri/            # desktop shell (requires system deps)
```

## tech stack

**backend** — rust, axum, tokio, bollard (docker), rusqlite, sysinfo  
**frontend** — react, typescript, vite, tailwind, cytoscape.js, zustand, tanstack query  
**desktop** — tauri (wip)

## development

```bash
# lint
cargo clippy --workspace -- -D warnings

# test
cargo test --workspace

# format
cargo fmt --all

# frontend only
cd apps/frontend && pnpm dev
```

## screenshots

*(coming soon)*

## license

mit
