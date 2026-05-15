import { useState } from 'react'
import { useSecurity } from '../hooks/use-queries'
import { Shield, ChevronDown, ChevronUp } from 'lucide-react'

type Severity = 'Critical' | 'High' | 'Medium' | 'Low' | 'Info'

const severityOrder: Severity[] = ['Critical', 'High', 'Medium', 'Low', 'Info']

const severityColor: Record<Severity, string> = {
  Critical: 'bg-red',
  High: 'bg-orange',
  Medium: 'bg-yellow',
  Low: 'bg-blue',
  Info: 'bg-zinc-500',
}

export function Security() {
  const { data: findings, isLoading } = useSecurity()
  const [expanded, setExpanded] = useState<string | null>(null)
  const [filter, setFilter] = useState<Severity | 'All'>('All')

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-pulse text-zinc-500">Analyzing security...</div>
      </div>
    )
  }

  const filtered =
    findings?.filter((f) => filter === 'All' || f.severity === filter) ?? []

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold">Security Findings</h1>
          <p className="text-sm text-zinc-500 mt-1">
            {findings?.length ?? 0} total findings
          </p>
        </div>
      </div>

      {/* Filter chips */}
      <div className="flex flex-wrap gap-2">
        {(['All', ...severityOrder] as const).map((s) => (
          <button
            key={s}
            onClick={() => setFilter(s)}
            className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors border ${
              filter === s
                ? 'bg-accent/10 border-accent/30 text-accent-hover'
                : 'bg-surface-3 border-zinc-700/50 text-zinc-400 hover:text-zinc-300'
            }`}
          >
            {s}
            {s !== 'All' && (
              <span className="ml-1.5 opacity-60">
                {findings?.filter((f) => f.severity === s).length}
              </span>
            )}
          </button>
        ))}
      </div>

      {/* Findings list */}
      <div className="space-y-2">
        {filtered.length === 0 ? (
          <div className="bg-surface-2 border border-zinc-800 rounded-xl p-8 text-center">
            <Shield size={32} className="mx-auto text-green mb-2" />
            <p className="text-sm text-zinc-400">No findings match this filter</p>
          </div>
        ) : (
          filtered.map((f) => (
            <div
              key={f.id}
              className="bg-surface-2 border border-zinc-800 rounded-xl overflow-hidden"
            >
              <button
                onClick={() => setExpanded(expanded === f.id ? null : f.id)}
                className="w-full flex items-center gap-3 p-4 text-left hover:bg-zinc-800/30 transition-colors"
              >
                <div
                  className={`w-2.5 h-2.5 rounded-full shrink-0 ${
                    severityColor[f.severity]
                  }`}
                />
                <div className="flex-1 min-w-0">
                  <div className="text-sm font-medium text-zinc-200 truncate">
                    {f.title}
                  </div>
                  <div className="text-xs text-zinc-500 truncate">
                    {f.description}
                  </div>
                </div>
                <span className="text-xs font-mono text-zinc-500 uppercase">
                  {f.severity}
                </span>
                {expanded === f.id ? (
                  <ChevronUp size={16} className="text-zinc-500" />
                ) : (
                  <ChevronDown size={16} className="text-zinc-500" />
                )}
              </button>

              {expanded === f.id && (
                <div className="px-4 pb-4 pt-0 border-t border-zinc-800/50">
                  <div className="mt-3 space-y-2 text-sm">
                    <div>
                      <span className="text-zinc-500">Source: </span>
                      <span className="text-zinc-300">{f.source}</span>
                    </div>
                    {f.remediation && (
                      <div>
                        <span className="text-zinc-500">Remediation: </span>
                        <span className="text-accent-hover">{f.remediation}</span>
                      </div>
                    )}
                    <div>
                      <span className="text-zinc-500">Description: </span>
                      <span className="text-zinc-400">{f.description}</span>
                    </div>
                  </div>
                </div>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  )
}
