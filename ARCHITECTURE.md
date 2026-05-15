# Monitaur Architecture

## Vision

Monitaur is a local-first infrastructure intelligence platform focused on:
- infrastructure visibility
- security posture analysis
- network intelligence
- privacy insights
- service relationship mapping
- elegant operational UX

It is NOT primarily a deployment platform.

The goal is to help users understand:
- what is running,
- what is exposed,
- how services communicate,
- what is risky,
- what leaks data,
- what can be hardened.

---

# Core Principles

## 1. Local-First
All infrastructure analysis happens locally by default.

No mandatory cloud dependency.

No telemetry.

No infrastructure metadata leaves the machine unless explicitly configured.

---

## 2. Read-Only First
Initial versions avoid infrastructure mutation.

Monitaur observes infrastructure before attempting orchestration or management.

This:
- reduces risk,
- simplifies trust,
- improves adoption.

---

## 3. Modular System Design
Every subsystem is isolated.

Core modules communicate through internal APIs/events.

This allows:
- future plugin support,
- independent scaling,
- cleaner testing,
- easier feature expansion.

---

## 4. Security-Centric
Security is not an optional module.

Every infrastructure component should expose:
- risk analysis,
- exposure visibility,
- hardening opportunities.

---

# High-Level System Architecture

+------------------------------------------------------+
|                    Frontend UI                       |
|            React + Tauri Desktop Client              |
+------------------------------------------------------+
|              Internal API Gateway Layer              |
+------------------------------------------------------+
|                                                      |
|   Discovery Engine     Monitoring Engine             |
|   Security Engine      Network Intelligence Engine   |
|   Visualization Engine Metadata Engine               |
|                                                      |
+------------------------------------------------------+
|                 Event Bus / Message Layer            |
+------------------------------------------------------+
|                                                      |
| Docker API     System APIs     Network APIs          |
| Containerd     ProcFS          Socket Inspection     |
| DNS APIs       SSL/TLS APIs    Reverse Proxy APIs    |
|                                                      |
+------------------------------------------------------+

---

# Frontend Architecture

## Stack

- React
- TypeScript
- Tauri
- Zustand
- React Query
- Tailwind
- D3.js or Cytoscape.js for topology visualization

---

# Frontend Responsibilities

## Infrastructure Dashboard
Displays:
- services
- containers
- health status
- exposure state
- metrics
- traffic summaries

---

## Network Topology View

Interactive graph visualization:
- nodes
- services
- databases
- proxies
- internet connections
- trust boundaries

Capabilities:
- zoom
- isolate
- dependency tracing
- relationship highlighting

---

## Security Dashboard

Displays:
- open ports
- exposed services
- insecure protocols
- outdated images
- suspicious outbound traffic
- weak TLS configurations

---

## Service Inspector

Detailed service analysis:
- ports
- environment variables
- mounted volumes
- network memberships
- reverse proxy relationships
- outbound destinations

---

# Backend Architecture

## Primary Language

Rust

Reasoning:
- concurrency
- memory safety
- low overhead
- networking performance
- systems integration
- native binaries

---

# Core Backend Modules

## 1. Discovery Engine

Responsible for infrastructure discovery.

### Responsibilities
- Docker container enumeration
- network scanning
- service detection
- reverse proxy detection
- port discovery
- DNS resolution

### Inputs
- Docker socket
- system interfaces
- local network APIs

### Outputs
- normalized infrastructure graph

---

## 2. Monitoring Engine

Real-time infrastructure monitoring.

### Responsibilities
- CPU metrics
- memory usage
- network throughput
- process health
- service uptime
- container lifecycle tracking

### Design
Uses asynchronous polling/event subscriptions.

---

## 3. Security Engine

Core infrastructure risk analysis.

### Responsibilities
- exposed port analysis
- weak TLS detection
- risky container configs
- root container detection
- privileged container detection
- secret exposure detection
- public attack surface mapping

### Future Expansion
- CVE analysis
- runtime anomaly detection
- intrusion heuristics

---

## 4. Network Intelligence Engine

Traffic and connectivity analysis.

### Responsibilities
- outbound destination tracking
- dependency graph generation
- DNS inspection
- external API detection
- telemetry identification
- traffic classification

---

## 5. Visualization Engine

Transforms infrastructure state into graph-compatible structures.

### Responsibilities
- topology generation
- node clustering
- relationship mapping
- graph optimization

---

## 6. Metadata Engine

Maintains normalized system state.

### Responsibilities
- caching
- entity relationships
- service indexing
- historical snapshots

---

# Internal Communication

## Event Bus Architecture

Modules communicate asynchronously.

Example events:
- container.started
- service.exposed
- suspicious.traffic.detected
- tls.certificate.expired

Benefits:
- loose coupling
- extensibility
- plugin compatibility
- future distributed support

---

# Data Model

## Core Entities

### Service
Represents:
- container
- process
- application

Fields:
- id
- name
- type
- ports
- networks
- health
- exposure_state

---

### NetworkNode

Represents:
- internal services
- external services
- domains
- endpoints

---

### SecurityFinding

Represents:
- severity
- source
- remediation
- timestamps

---

# Infrastructure Graph Model

Monitaur internally maintains:
- directed service graph
- dependency graph
- exposure graph

This powers:
- topology views
- risk analysis
- path tracing

---

# Plugin System (Future)

Plugins should run in isolated sandboxes.

Capabilities:
- custom analyzers
- protocol parsers
- integrations
- exporters

Possible plugin APIs:
- WASM
- Lua
- gRPC subprocesses

WASM is preferred long-term.

---

# Persistence Layer

## Initial Phase
SQLite

Stores:
- snapshots
- findings
- topology states
- historical metrics

---

## Future Expansion
Optional:
- ClickHouse
- TimescaleDB

for long-term analytics.

---

# Security Model

## Trust Assumptions

Monitaur has privileged visibility.

Therefore:
- no cloud dependency
- minimal outbound requests
- signed releases
- reproducible builds preferred
- transparent permissions

---

# Initial MVP Scope

## Included
- Docker discovery
- topology mapping
- port analysis
- exposure detection
- reverse proxy detection
- live dashboard
- security findings
- local persistence

---

## Explicitly Excluded
- orchestration
- deployment pipelines
- Kubernetes management
- cloud hosting
- remote execution
- agent-based architecture

These increase complexity significantly.

---

# Long-Term Vision

Monitaur evolves into:
- infrastructure intelligence layer
- local-first observability platform
- security visibility system
- trust mapping engine

Potential future capabilities:
- zero trust policy visualization
- attack path simulation
- runtime anomaly detection
- infrastructure hardening automation
- distributed node federation
- AI-assisted infrastructure analysis

---

# Recommended Development Order

## Phase 1
Infrastructure discovery engine

## Phase 2
Topology visualization

## Phase 3
Monitoring system

## Phase 4
Security analysis engine

## Phase 5
Historical persistence

## Phase 6
Advanced network intelligence

---

# Design Philosophy

Monitaur should feel:
- calm
- powerful
- trustworthy
- elegant
- transparent

Not:
- enterprise-heavy
- cluttered
- intimidating
- noisy

The interface should prioritize:
- clarity,
- visibility,
- explainability.
