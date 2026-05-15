import type {
  InfraGraph,
  SecurityFinding,
  NetworkAnalysis,
  TopologyGraph,
  Service,
  MetricsResponse,
  ScanResponse,
} from './types'

const BASE = '/api'

async function fetchJson<T>(url: string): Promise<T> {
  const res = await fetch(url)
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }))
    throw new Error(err.error || `Request failed: ${res.status}`)
  }
  return res.json()
}

export const api = {
  health: () => fetchJson<{ status: string; version: string }>(`${BASE}/health`),

  scan: () => fetchJson<ScanResponse>(`${BASE}/scan`),

  services: () => fetchJson<Service[]>(`${BASE}/services`),

  service: (id: string) => fetchJson<Service>(`${BASE}/services/${encodeURIComponent(id)}`),

  metrics: () => fetchJson<MetricsResponse>(`${BASE}/metrics`),

  security: () => fetchJson<SecurityFinding[]>(`${BASE}/security`),

  network: () => fetchJson<NetworkAnalysis>(`${BASE}/network`),

  visualization: () => fetchJson<TopologyGraph>(`${BASE}/visualization`),
}
