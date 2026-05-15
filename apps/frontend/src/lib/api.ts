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

function getToken(): string | null {
  return localStorage.getItem('monitaur_token')
}

function clearToken() {
  localStorage.removeItem('monitaur_token')
}

function setToken(token: string) {
  localStorage.setItem('monitaur_token', token)
}

async function fetchJson<T>(url: string, options?: RequestInit): Promise<T> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  }
  const token = getToken()
  if (token) {
    headers['Authorization'] = `Bearer ${token}`
  }

  const res = await fetch(url, { ...options, headers })

  if (res.status === 401) {
    clearToken()
    window.dispatchEvent(new CustomEvent('monitaur:unauthorized'))
    throw new Error('Unauthorized')
  }

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

  // Auth
  authStatus: () => fetchJson<{ has_admin: boolean; auth_enabled: boolean }>(`${BASE}/auth/status`),

  setup: (password: string) =>
    fetchJson<{ token: string; message: string }>(`${BASE}/auth/setup`, {
      method: 'POST',
      body: JSON.stringify({ password }),
    }),

  login: (password: string) =>
    fetchJson<{ token: string; message: string }>(`${BASE}/auth/login`, {
      method: 'POST',
      body: JSON.stringify({ password }),
    }),

  // Token management
  getToken,
  setToken,
  clearToken,
}
