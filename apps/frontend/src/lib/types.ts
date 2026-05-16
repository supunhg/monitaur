// ── Matching monitaur-core Rust models ─────────────────────────

export interface Service {
  id: string
  name: string
  image: string | null
  service_type: 'Container' | 'Process' | 'Application'
  class: 'Database' | 'Cache' | 'ReverseProxy' | 'WebApp' | 'Worker' | 'Messaging' | 'Monitoring' | 'Security' | 'Utility' | 'Unknown'
  ports: Port[]
  networks: string[]
  health: 'Healthy' | 'Degraded' | 'Unhealthy' | 'Unknown'
  status: string
  labels: Record<string, string>
  exposure_state: 'Exposed' | 'Internal' | 'Unknown'
}

export interface Port {
  port: number
  protocol: 'Tcp' | 'Udp'
  exposed: boolean
}

export interface NetworkNode {
  id: string
  kind: 'InternalService' | 'ExternalService' | 'Domain' | 'Endpoint'
  addresses: string[]
}

export interface Edge {
  source_id: string
  target_id: string
  relation: 'DependsOn' | 'Exposes' | 'ConnectsTo' | 'Contains'
}

export interface InfraGraph {
  services: Service[]
  network_nodes: NetworkNode[]
  edges: Edge[]
}

export interface SecurityFinding {
  id: string
  severity: 'Critical' | 'High' | 'Medium' | 'Low' | 'Info'
  title: string
  description: string
  source: string
  remediation: string | null
  timestamp: string
}

export interface SystemMetrics {
  cpu_percent: number
  memory_total_bytes: number
  memory_used_bytes: number
  memory_percent: number
  network_rx_bytes: number
  network_tx_bytes: number
  timestamp: string
}

export interface ContainerMetrics {
  container_id: string
  cpu_percent: number
  memory_usage_bytes: number
  memory_limit_bytes: number
  memory_percent: number
  network_rx_bytes: number
  network_tx_bytes: number
  pids_current: number | null
  timestamp: string
}

export interface Connection {
  local_addr: string
  local_port: number
  remote_addr: string
  remote_port: number
  state: string
  inode: number
  pid: number | null
  container_id: string | null
}

export interface TrafficFlow {
  source: string
  destination: string
  port: number
  class: string
  connection_count: number
}

export interface DnsQuery {
  query: string
  query_type: string
  response: string[]
}

export interface NetworkAnalysis {
  connections: Connection[]
  flows: TrafficFlow[]
  dns_queries: DnsQuery[]
}

export interface TopologyNode {
  id: string
  label: string
  node_type: string
  group: string
  layer: number
  x: number
  y: number
  metadata: [string, string][]
}

export interface TopologyEdge {
  id: string
  source: string
  target: string
  label: string
  edge_type: string
  width: number
}

export interface NodeGroup {
  id: string
  label: string
  node_ids: string[]
  group_type: string
}

export interface TopologyGraph {
  nodes: TopologyNode[]
  edges: TopologyEdge[]
  groups: NodeGroup[]
  layers: string[]
}

export interface MetricsResponse {
  system: SystemMetrics | null
  containers: ContainerMetrics[]
}

export interface MetricsSnapshot {
  system: SystemMetrics | null
  containers: ContainerMetrics[]
  timestamp: string
}

export interface ScanResponse {
  discovery: InfraGraph
  security: SecurityFinding[]
  network: NetworkAnalysis
  visualization: TopologyGraph
}
