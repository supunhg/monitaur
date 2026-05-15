import { useScan } from '../hooks/use-queries'

export function Topology() {
  const { data, isLoading } = useScan()

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-pulse text-zinc-500">Loading topology...</div>
      </div>
    )
  }

  if (!data) return null

  const { visualization } = data

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold">Topology</h1>
        <p className="text-sm text-zinc-500 mt-1">
          {visualization.nodes.length} nodes · {visualization.edges.length} edges ·{' '}
          {visualization.groups.length} groups
        </p>
      </div>

      <div className="bg-surface-2 border border-zinc-800 rounded-xl p-5 min-h-[400px] flex items-center justify-center">
        <div className="text-center space-y-3">
          <Network3D />
          <p className="text-sm text-zinc-500">
            Interactive topology graph will render here (Cytoscape.js).
          </p>
          <p className="text-xs text-zinc-600">
            {visualization.layers.length} layers available
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Layers */}
        <div className="bg-surface-2 border border-zinc-800 rounded-xl p-5 space-y-3">
          <h2 className="text-sm font-medium text-zinc-300">Layers</h2>
          <div className="space-y-2">
            {visualization.layers.map((layer, i) => {
              const count = visualization.nodes.filter((n) => n.layer === i).length
              return (
                <div key={i} className="flex items-center justify-between text-sm">
                  <span className="text-zinc-400">{layer}</span>
                  <span className="text-zinc-500">{count} nodes</span>
                </div>
              )
            })}
          </div>
        </div>

        {/* Groups */}
        <div className="bg-surface-2 border border-zinc-800 rounded-xl p-5 space-y-3">
          <h2 className="text-sm font-medium text-zinc-300">Groups</h2>
          <div className="space-y-2 max-h-64 overflow-y-auto">
            {visualization.groups.map((g) => (
              <div
                key={g.id}
                className="flex items-center justify-between text-sm"
              >
                <span className="text-zinc-400">{g.label}</span>
                <span className="text-zinc-500">{g.node_ids.length} nodes</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}

function Network3D() {
  return (
    <svg width="80" height="80" viewBox="0 0 80 80" fill="none" className="mx-auto text-accent/40">
      <circle cx="40" cy="20" r="6" stroke="currentColor" strokeWidth="1.5" />
      <circle cx="20" cy="50" r="6" stroke="currentColor" strokeWidth="1.5" />
      <circle cx="60" cy="50" r="6" stroke="currentColor" strokeWidth="1.5" />
      <circle cx="40" cy="65" r="6" stroke="currentColor" strokeWidth="1.5" />
      <line x1="36" y1="24" x2="24" y2="46" stroke="currentColor" strokeWidth="1" opacity="0.4" />
      <line x1="44" y1="24" x2="56" y2="46" stroke="currentColor" strokeWidth="1" opacity="0.4" />
      <line x1="24" y1="54" x2="36" y2="61" stroke="currentColor" strokeWidth="1" opacity="0.4" />
      <line x1="56" y1="54" x2="44" y2="61" stroke="currentColor" strokeWidth="1" opacity="0.4" />
      <line x1="40" y1="26" x2="40" y2="59" stroke="currentColor" strokeWidth="1" opacity="0.4" />
    </svg>
  )
}
