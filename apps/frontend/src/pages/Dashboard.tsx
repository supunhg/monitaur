import { useScan, useMetrics, useNetwork, useMetricsHistory } from '../hooks/use-queries'
import { bytesToHuman } from '../lib/utils'
import { Cpu, HardDrive, Network, Shield, Server, Activity, AlertTriangle, TrendingUp } from 'lucide-react'
import {
  LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer,
} from 'recharts'

export function Dashboard() {
  const scan = useScan()
  const metrics = useMetrics(true)
  const network = useNetwork(true)

  if (scan.isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-pulse text-zinc-500">Scanning infrastructure...</div>
      </div>
    )
  }

  if (scan.error) {
    return (
      <div className="bg-surface-2 border border-red/20 rounded-xl p-8 text-center space-y-3">
        <AlertTriangle size={40} className="mx-auto text-red" />
        <p className="text-sm text-red">Failed to connect to API</p>
        <p className="text-xs text-zinc-500">{(scan.error as Error).message}</p>
        <button
          onClick={() => scan.refetch()}
          className="inline-flex items-center gap-2 px-4 py-2 text-sm bg-accent/10 border border-accent/30 text-accent-hover rounded-lg hover:bg-accent/20 transition-colors"
        >
          Retry
        </button>
      </div>
    )
  }

  if (!scan.data) return null

  const { discovery, security, visualization } = scan.data
  const netAnalysis = network.data
  const sysMetrics = metrics.data?.system
  const metricsError = metrics.error
  const networkError = network.error

  const cards = [
    {
      label: 'Services',
      value: discovery.services.length.toString(),
      sub: `${discovery.network_nodes.length} networks`,
      icon: Server,
      color: 'text-blue',
    },
    {
      label: 'Security Findings',
      value: security.length.toString(),
      sub: `${security.filter((s) => s.severity === 'High' || s.severity === 'Critical').length} critical/high`,
      icon: Shield,
      color: security.length > 0 ? 'text-red' : 'text-green',
    },
    {
      label: 'CPU',
      value: sysMetrics ? `${sysMetrics.cpu_percent.toFixed(1)}%` : metricsError ? 'Error' : '—',
      sub: metricsError ? 'Connection failed' : 'current usage',
      icon: Cpu,
      color: metricsError ? 'text-red' : 'text-accent-hover',
    },
    {
      label: 'Memory',
      value: sysMetrics ? `${sysMetrics.memory_percent.toFixed(1)}%` : metricsError ? 'Error' : '—',
      sub: sysMetrics
        ? `${bytesToHuman(sysMetrics.memory_used_bytes)} / ${bytesToHuman(sysMetrics.memory_total_bytes)}`
        : metricsError
          ? 'Connection failed'
          : '',
      icon: HardDrive,
      color: metricsError ? 'text-red' : 'text-yellow',
    },
  ]

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold">Dashboard</h1>
          <p className="text-sm text-zinc-500 mt-1">Infrastructure overview</p>
        </div>
        <div className="flex items-center gap-2 text-xs text-zinc-500">
          <Activity size={14} />
          <span>Live</span>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {cards.map((card) => (
          <div
            key={card.label}
            className="bg-surface-2 border border-zinc-800 rounded-xl p-5 space-y-3"
          >
            <div className="flex items-center justify-between">
              <span className="text-sm text-zinc-400">{card.label}</span>
              <card.icon size={20} className={card.color} />
            </div>
            <div className="text-2xl font-bold">{card.value}</div>
            <div className="text-xs text-zinc-500">{card.sub}</div>
          </div>
        ))}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="bg-surface-2 border border-zinc-800 rounded-xl p-5 space-y-3">
          <h2 className="text-sm font-medium text-zinc-300">Live System Metrics</h2>
          {metricsError ? (
            <p className="text-sm text-red">
              Metrics unavailable: {(metricsError as Error).message}
            </p>
          ) : sysMetrics ? (
            <div className="space-y-3">
              <MetricBar label="CPU" value={sysMetrics.cpu_percent} color="bg-accent-hover" />
              <MetricBar label="Memory" value={sysMetrics.memory_percent} color="bg-yellow" />
              <div className="flex justify-between text-xs">
                <span className="text-zinc-500">Network ↓</span>
                <span className="text-zinc-300 font-mono">
                  {bytesToHuman(sysMetrics.network_rx_bytes)}
                </span>
              </div>
              <div className="flex justify-between text-xs">
                <span className="text-zinc-500">Network ↑</span>
                <span className="text-zinc-300 font-mono">
                  {bytesToHuman(sysMetrics.network_tx_bytes)}
                </span>
              </div>
            </div>
          ) : (
            <p className="text-sm text-zinc-500">Waiting for metrics...</p>
          )}
        </div>

        <div className="bg-surface-2 border border-zinc-800 rounded-xl p-5 space-y-3">
          <h2 className="text-sm font-medium text-zinc-300">Recent Security Findings</h2>
          {security.length === 0 ? (
            <p className="text-sm text-zinc-500">No findings</p>
          ) : (
            <div className="space-y-2 max-h-64 overflow-y-auto">
              {security.slice(0, 10).map((f) => (
                <div key={f.id} className="flex items-start gap-3 text-sm">
                  <span
                    className={`mt-0.5 w-2 h-2 rounded-full shrink-0 ${
                      f.severity === 'Critical' || f.severity === 'High'
                        ? 'bg-red'
                        : 'bg-yellow'
                    }`}
                  />
                  <div>
                    <div className="text-zinc-300">{f.title}</div>
                    <div className="text-zinc-500 text-xs truncate">{f.description}</div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="bg-surface-2 border border-zinc-800 rounded-xl p-5 space-y-3">
          <h2 className="text-sm font-medium text-zinc-300">Live Connections</h2>
          {networkError ? (
            <p className="text-sm text-red">
              Network data unavailable: {(networkError as Error).message}
            </p>
          ) : !netAnalysis || netAnalysis.connections.length === 0 ? (
            <p className="text-sm text-zinc-500">No active connections</p>
          ) : (
            <div className="space-y-2 max-h-64 overflow-y-auto">
              {netAnalysis.connections.slice(0, 10).map((c, i) => (
                <div key={i} className="flex items-center gap-2 text-sm">
                  <div className="w-2 h-2 rounded-full bg-green shrink-0" />
                  <span className="text-zinc-400 font-mono text-xs">
                    {c.local_addr}:{c.local_port}
                  </span>
                  <span className="text-zinc-600">→</span>
                  <span className="text-zinc-300 font-mono text-xs">
                    {c.remote_addr}:{c.remote_port}
                  </span>
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="bg-surface-2 border border-zinc-800 rounded-xl p-5 space-y-3">
          <h2 className="text-sm font-medium text-zinc-300">Services by Class</h2>
          <div className="flex flex-wrap gap-2">
            {discovery.services
              .reduce(
                (acc, s) => {
                  const existing = acc.find((a) => a.class === s.class)
                  if (existing) existing.count++
                  else acc.push({ class: s.class, count: 1 })
                  return acc
                },
                [] as { class: string; count: number }[],
              )
              .map((g) => (
                <div
                  key={g.class}
                  className="bg-surface-3 border border-zinc-700/50 rounded-lg px-3 py-1.5 text-sm"
                >
                  <span className="text-zinc-400">{g.class}</span>
                  <span className="text-zinc-600 ml-2">{g.count}</span>
                </div>
              ))}
          </div>
        </div>
      </div>

      <HistoryChart />
    </div>
  )
}

function MetricBar({ label, value, color }: { label: string; value: number; color: string }) {
  return (
    <div>
      <div className="flex justify-between text-xs mb-1">
        <span className="text-zinc-500">{label}</span>
        <span className="text-zinc-300 font-mono">{value.toFixed(1)}%</span>
      </div>
      <div className="w-full h-1.5 bg-zinc-800 rounded-full overflow-hidden">
        <div
          className={`h-full rounded-full transition-all duration-500 ${color}`}
          style={{ width: `${Math.min(value, 100)}%` }}
        />
      </div>
    </div>
  )
}

function HistoryChart() {
  const { data: history } = useMetricsHistory()

  if (!history || history.length < 2) {
    return (
      <div className="bg-surface-2 border border-zinc-800 rounded-xl p-5 space-y-3">
        <div className="flex items-center gap-2">
          <TrendingUp size={16} className="text-accent-hover" />
          <h2 className="text-sm font-medium text-zinc-300">CPU & Memory History</h2>
        </div>
        <p className="text-xs text-zinc-500">
          {history && history.length === 1
            ? 'Collecting data... check back after the next poll cycle.'
            : 'No historical data yet. Metrics are stored every poll cycle.'}
        </p>
      </div>
    )
  }

  const data = history
    .map((s) => ({
      time: new Date(s.timestamp).toLocaleTimeString(),
      cpu: s.system ? Math.round(s.system.cpu_percent * 10) / 10 : 0,
      memory: s.system ? Math.round(s.system.memory_percent * 10) / 10 : 0,
    }))
    .reverse()

  return (
    <div className="bg-surface-2 border border-zinc-800 rounded-xl p-5 space-y-4">
      <div className="flex items-center gap-2">
        <TrendingUp size={16} className="text-accent-hover" />
        <h2 className="text-sm font-medium text-zinc-300">CPU & Memory History</h2>
        <span className="text-xs text-zinc-500 ml-auto">{data.length} data points</span>
      </div>
      <div className="h-48">
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={data}>
            <CartesianGrid strokeDasharray="3 3" stroke="#2a2a3e" />
            <XAxis dataKey="time" tick={{ fontSize: 10, fill: '#71717a' }} interval="preserveStartEnd" />
            <YAxis tick={{ fontSize: 10, fill: '#71717a' }} domain={[0, 'auto']} />
            <Tooltip
              contentStyle={{ background: '#14141f', border: '1px solid #2a2a3e', borderRadius: 8, fontSize: 12 }}
              labelStyle={{ color: '#e4e4e7' }}
            />
            <Line type="monotone" dataKey="cpu" stroke="#818cf8" strokeWidth={2} dot={false} name="CPU %" />
            <Line type="monotone" dataKey="memory" stroke="#eab308" strokeWidth={2} dot={false} name="Memory %" />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  )
}
