import { useScan } from '../hooks/use-queries'
import { bytesToHuman } from '../lib/utils'
import { Cpu, MemoryStick, Network, Shield, Server } from 'lucide-react'

export function Dashboard() {
  const { data, isLoading, error } = useScan()

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-pulse text-zinc-500">Scanning infrastructure...</div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-red">Error: {(error as Error).message}</div>
      </div>
    )
  }

  if (!data) return null

  const { discovery, security, network, visualization } = data

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
      label: 'Network Flows',
      value: network.flows.length.toString(),
      sub: `${network.connections.length} active connections`,
      icon: Network,
      color: 'text-accent-hover',
    },
    {
      label: 'Topology',
      value: visualization.nodes.length.toString(),
      sub: `${visualization.edges.length} edges`,
      icon: Cpu,
      color: 'text-yellow',
    },
  ]

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold">Dashboard</h1>
        <p className="text-sm text-zinc-500 mt-1">Infrastructure overview</p>
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
        {/* Security Findings */}
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

        {/* Network Activity */}
        <div className="bg-surface-2 border border-zinc-800 rounded-xl p-5 space-y-3">
          <h2 className="text-sm font-medium text-zinc-300">Network Activity</h2>
          {network.connections.length === 0 ? (
            <p className="text-sm text-zinc-500">No active connections</p>
          ) : (
            <div className="space-y-2 max-h-64 overflow-y-auto">
              {network.connections.slice(0, 10).map((c, i) => (
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
      </div>

      {/* Service Classes */}
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
  )
}
