import { useScan } from '../hooks/use-queries'
import { CytoscapeGraph } from '../components/CytoscapeGraph'

export function Topology() {
  const { data, isLoading, error } = useScan()

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-pulse text-zinc-500">Loading topology...</div>
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

  const { visualization } = data

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold">Topology</h1>
          <p className="text-sm text-zinc-500 mt-1">
            {visualization.nodes.length} nodes · {visualization.edges.length} edges ·{' '}
            {visualization.groups.length} groups
          </p>
        </div>
      </div>

      <CytoscapeGraph graph={visualization} />

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Layers */}
        <div className="bg-surface-2 border border-zinc-800 rounded-xl p-5 space-y-3">
          <h2 className="text-sm font-medium text-zinc-300">Layers</h2>
          <div className="space-y-2">
            {visualization.layers.map((layer, i) => {
              const count = visualization.nodes.filter((n) => n.layer === i).length
              return (
                <div key={i} className="flex items-center justify-between text-sm py-1">
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
                className="flex items-center justify-between text-sm py-1"
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
