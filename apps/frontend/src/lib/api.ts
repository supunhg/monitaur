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
let apiBasePromise: Promise<string> | null = null

async function resolveApiBase(): Promise<string> {
  if (!isTauri()) return apiBase
  if (!apiBasePromise) {
    apiBasePromise = (async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      for (let attempt = 0; attempt < 20; attempt++) {
        const port = await invoke<number>('get_api_port')
        if (port > 0) {
          apiBase = `http://127.0.0.1:${port}/api`
          return apiBase
        }
        await new Promise((resolve) => window.setTimeout(resolve, 100))
      }
      throw new Error('Tauri API port is not ready')
    })()
  }
  return apiBasePromise
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
  const base = await resolveApiBase()
  const path = `${base}${url}`
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options?.headers as Record<string, string> | undefined),
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
