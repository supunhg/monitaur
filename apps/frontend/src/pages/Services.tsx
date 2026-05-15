import { useState } from 'react'
import { useScan, useService } from '../hooks/use-queries'
import { Server, Search, X } from 'lucide-react'

export function Services() {
  const { data, isLoading } = useScan()
  const [search, setSearch] = useState('')
  const [selectedId, setSelectedId] = useState<string | null>(null)

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-pulse text-zinc-500">Loading services...</div>
      </div>
    )
  }

  const services = data?.discovery.services ?? []

  const filtered = services.filter(
    (s) =>
      s.name.toLowerCase().includes(search.toLowerCase()) ||
      s.class.toLowerCase().includes(search.toLowerCase()) ||
      (s.image && s.image.toLowerCase().includes(search.toLowerCase())),
  )

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold">Services</h1>
        <p className="text-sm text-zinc-500 mt-1">{services.length} total</p>
      </div>

      {/* Search */}
      <div className="relative">
        <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500" />
        <input
          type="text"
          placeholder="Search by name, class, or image..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="w-full bg-surface-2 border border-zinc-800 rounded-lg pl-10 pr-10 py-2.5 text-sm text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-accent/50 transition-colors"
        />
        {search && (
          <button
            onClick={() => setSearch('')}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-zinc-500 hover:text-zinc-300"
          >
            <X size={16} />
          </button>
        )}
      </div>

      {/* Service cards */}
      <div className="space-y-2">
        {filtered.length === 0 ? (
          <div className="bg-surface-2 border border-zinc-800 rounded-xl p-8 text-center">
            <Server size={32} className="mx-auto text-zinc-600 mb-2" />
            <p className="text-sm text-zinc-500">No services match your search</p>
          </div>
        ) : (
          filtered.map((s) => (
            <button
              key={s.id}
              onClick={() => setSelectedId(selectedId === s.id ? null : s.id)}
              className={`w-full text-left bg-surface-2 border rounded-xl p-4 transition-colors hover:bg-zinc-800/30 ${
                selectedId === s.id ? 'border-accent/30' : 'border-zinc-800'
              }`}
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3 min-w-0">
                  <div
                    className={`w-2 h-2 rounded-full shrink-0 ${
                      s.health === 'Healthy'
                        ? 'bg-green'
                        : s.health === 'Degraded'
                          ? 'bg-yellow'
                          : s.health === 'Unhealthy'
                            ? 'bg-red'
                            : 'bg-zinc-500'
                    }`}
                  />
                  <div className="min-w-0">
                    <div className="text-sm font-medium text-zinc-200 truncate">
                      {s.name}
                    </div>
                    <div className="text-xs text-zinc-500 truncate">
                      {s.class}
                      {s.image && ` · ${s.image}`}
                    </div>
                  </div>
                </div>
                <div className="flex items-center gap-3 text-xs text-zinc-500">
                  <span>{s.ports.length} ports</span>
                  <span>{s.networks.length} networks</span>
                  <span className="uppercase">{s.status}</span>
                </div>
              </div>

              {selectedId === s.id && (
                <ServiceDetail serviceId={s.id} />
              )}
            </button>
          ))
        )}
      </div>
    </div>
  )
}

function ServiceDetail({ serviceId }: { serviceId: string }) {
  const { data: service } = useService(serviceId)

  if (!service) return null

  return (
    <div className="mt-4 pt-4 border-t border-zinc-800/50 space-y-4">
      {/* Ports */}
      <div>
        <h3 className="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-2">
          Ports
        </h3>
        <div className="flex flex-wrap gap-2">
          {service.ports.length === 0 && (
            <span className="text-xs text-zinc-600">No ports</span>
          )}
          {service.ports.map((p, i) => (
            <div
              key={i}
              className={`px-2.5 py-1 rounded-lg text-xs font-mono border ${
                p.exposed
                  ? 'bg-yellow/5 border-yellow/20 text-yellow'
                  : 'bg-zinc-800/50 border-zinc-700/50 text-zinc-400'
              }`}
            >
              {p.port}/{p.protocol}
              {p.exposed && ' (exposed)'}
            </div>
          ))}
        </div>
      </div>

      {/* Networks */}
      <div>
        <h3 className="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-2">
          Networks
        </h3>
        <div className="flex flex-wrap gap-2">
          {service.networks.length === 0 && (
            <span className="text-xs text-zinc-600">None</span>
          )}
          {service.networks.map((n) => (
            <span
              key={n}
              className="px-2.5 py-1 rounded-lg text-xs bg-zinc-800/50 border border-zinc-700/50 text-zinc-400 font-mono"
            >
              {n}
            </span>
          ))}
        </div>
      </div>

      {/* Labels */}
      {Object.keys(service.labels).length > 0 && (
        <div>
          <h3 className="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-2">
            Labels
          </h3>
          <div className="space-y-1">
            {Object.entries(service.labels).map(([k, v]) => (
              <div key={k} className="text-xs text-zinc-500">
                <span className="text-zinc-400">{k}</span>
                <span className="text-zinc-600">: </span>
                <span className="text-zinc-500">{v}</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}
