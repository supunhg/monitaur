import type {
  InfraGraph,
  SecurityFinding,
  NetworkAnalysis,
  TopologyGraph,
  Service,
  MetricsResponse,
  MetricsSnapshot,
  ScanResponse,
} from './types'

// ── Tauri detection ────────────────────────────────────────────

function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

let apiBase = '/api' // default for Vite proxy

// In Tauri mode, discover the API port from the backend
if (isTauri()) {
  // Dynamic import to avoid breaking in pure browser dev
  import('@tauri-apps/api/core').then(({ invoke }) => {
    invoke<number>('get_api_port').then((port) => {
      apiBase = `http://127.0.0.1:${port}/api`
    })
  })
}

// ── Token management ───────────────────────────────────────────

function getToken(): string | null {
  try {
    return localStorage.getItem('monitaur_token')
  } catch {
    return null
  }
}

function clearToken() {
  try {
    localStorage.removeItem('monitaur_token')
  } catch {
    // localStorage not available
  }
}

function setToken(token: string) {
  try {
    localStorage.setItem('monitaur_token', token)
  } catch {
    // localStorage not available
  }
}

// ── Core fetch ─────────────────────────────────────────────────

async function fetchJson<T>(url: string, options?: RequestInit): Promise<T> {
  const path = `${apiBase}${url}`
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  }
  const token = getToken()
  if (token) {
    headers['Authorization'] = `Bearer ${token}`
  }

  const res = await fetch(path, { ...options, headers })

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

// ── API methods ────────────────────────────────────────────────

export const api = {
  health: () => fetchJson<{ status: string; version: string }>('/health'),

  scan: () => fetchJson<ScanResponse>('/scan'),

  services: () => fetchJson<Service[]>('/services'),

  service: (id: string) => fetchJson<Service>(`/services/${encodeURIComponent(id)}`),

  metrics: () => fetchJson<MetricsResponse>('/metrics'),

  security: () => fetchJson<SecurityFinding[]>('/security'),

  findingsHistory: () => fetchJson<SecurityFinding[]>('/security/findings'),

  metricsHistory: () => fetchJson<MetricsSnapshot[]>('/metrics/history'),

  network: () => fetchJson<NetworkAnalysis>('/network'),

  visualization: () => fetchJson<TopologyGraph>('/visualization'),

  // Auth
  authStatus: () =>
    fetchJson<{ has_admin: boolean; auth_enabled: boolean }>('/auth/status'),

  setup: (password: string) =>
    fetchJson<{ token: string; message: string }>('/auth/setup', {
      method: 'POST',
      body: JSON.stringify({ password }),
    }),

  login: (password: string) =>
    fetchJson<{ token: string; message: string }>('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ password }),
    }),

  getToken,
  setToken,
  clearToken,
}
