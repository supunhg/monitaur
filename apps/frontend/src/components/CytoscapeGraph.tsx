import { useEffect, useRef, useState, useCallback } from 'react'
import cytoscape, { type Core, type EventObject } from 'cytoscape'
import type { TopologyGraph, TopologyNode } from '../lib/types'
import { X } from 'lucide-react'

const CLASS_COLORS: Record<string, string> = {
  Database: '#22c55e',
  Cache: '#3b82f6',
  ReverseProxy: '#f97316',
  WebApp: '#a78bfa',
  Worker: '#6366f1',
  Messaging: '#06b6d4',
  Monitoring: '#eab308',
  Security: '#ef4444',
  Unknown: '#71717a',
}

const EDGE_COLORS: Record<string, string> = {
  Exposes: '#ef4444',
  ConnectsTo: '#6366f1',
  DependsOn: '#22c55e',
  Contains: '#eab308',
}

interface Props {
  graph: TopologyGraph
}

export function CytoscapeGraph({ graph }: Props) {
  const containerRef = useRef<HTMLDivElement>(null)
  const cyRef = useRef<Core | null>(null)
  const [selectedNode, setSelectedNode] = useState<TopologyNode | null>(null)
  const [layout, setLayout] = useState<'preset' | 'breadthfirst' | 'concentric'>('preset')

  const buildElements = useCallback(() => {
    const elements: cytoscape.ElementDefinition[] = []

    for (const node of graph.nodes) {
      const color = CLASS_COLORS[node.node_type] || '#71717a'
      elements.push({
        data: {
          id: node.id,
          label: node.label,
          type: node.node_type,
          layer: node.layer,
          metadata: node.metadata,
        },
        position: { x: node.x + 300, y: node.y + 50 },
        classes: node.node_type.toLowerCase(),
      })
    }

    for (const edge of graph.edges) {
      elements.push({
        data: {
          id: edge.id,
          source: edge.source,
          target: edge.target,
          label: edge.edge_type,
          width: edge.width,
        },
        classes: edge.edge_type.toLowerCase(),
      })
    }

    return elements
  }, [graph])

  useEffect(() => {
    if (!containerRef.current) return

    const elements = buildElements()

    const cy = cytoscape({
      container: containerRef.current,
      elements,
      style: [
        {
          selector: 'node',
          style: {
            'background-color': (ele) => {
              const type = ele.data('type') || 'Unknown'
              return CLASS_COLORS[type] || '#71717a'
            },
            label: 'data(label)',
            'font-size': '11px',
            color: '#e4e4e7',
            'text-valign': 'bottom',
            'text-halign': 'center',
            'text-margin-y': 8,
            width: 28,
            height: 28,
            'font-family': 'Inter, sans-serif',
            'border-width': 2,
            'border-color': (ele) => {
              const type = ele.data('type') || 'Unknown'
              return CLASS_COLORS[type] || '#71717a'
            },
            'border-opacity': 0.5,
          },
        },
        {
          selector: 'edge',
          style: {
            width: (ele) => ele.data('width') || 1.5,
            'line-color': (ele) => {
              const type = ele.data('label') || ''
              return EDGE_COLORS[type] || '#3f3f46'
            },
            'target-arrow-color': (ele) => {
              const type = ele.data('label') || ''
              return EDGE_COLORS[type] || '#3f3f46'
            },
            'target-arrow-shape': 'triangle',
            'arrow-scale': 0.7,
            'curve-style': 'bezier',
            opacity: 0.6,
          },
        },
        {
          selector: 'node:selected',
          style: {
            'border-width': 3,
            'border-color': '#818cf8',
            'border-opacity': 1,
            'shadow-blur': 12,
            'shadow-color': '#818cf8',
            'shadow-opacity': 0.4,
          },
        },
        {
          selector: 'edge:selected',
          style: {
            opacity: 1,
            width: 3,
          },
        },
        {
          selector: '.highlighted',
          style: {
            opacity: 1,
          },
        },
        {
          selector: '.faded',
          style: {
            opacity: 0.15,
          },
        },
      ],
      layout: { name: 'preset' },
      wheelSensitivity: 0.4,
      minZoom: 0.2,
      maxZoom: 3,
      panningEnabled: true,
      userPanningEnabled: true,
      userZoomingEnabled: true,
    } as cytoscape.CytoscapeOptions)

    cyRef.current = cy

    // Node click handler
    cy.on('tap', 'node', (evt: EventObject) => {
      const nodeId = evt.target.id()
      const topoNode = graph.nodes.find((n) => n.id === nodeId)
      setSelectedNode(topoNode || null)
    })

    // Click background to deselect
    cy.on('tap', (evt: EventObject) => {
      if (evt.target === cy) {
        setSelectedNode(null)
      }
    })

    // Hover highlight
    cy.on('mouseover', 'node', (evt: EventObject) => {
      const node = evt.target
      node.addClass('highlighted')
      const neighbors = node.neighborhood().nodes()
      node.addClass('highlighted')
      neighbors.addClass('highlighted')
      cy.elements().not(node).not(neighbors).addClass('faded')
    })

    cy.on('mouseout', 'node', () => {
      cy.elements().removeClass('highlighted faded')
    })

    return () => {
      cy.destroy()
      cyRef.current = null
    }
  }, [graph, buildElements])

  const runLayout = (name: string) => {
    if (!cyRef.current) return
    setLayout(name as typeof layout)
    cyRef.current.layout({
      name,
      animate: true,
      animationDuration: 500,
      spacingFactor: 1.5,
      directed: true,
      roots: cyRef.current.nodes('[layer = 0]'),
    } as any).run()
  }

  const fitToScreen = () => {
    cyRef.current?.fit(undefined, 50)
  }

  return (
    <div className="relative">
      {/* Toolbar */}
      <div className="absolute top-3 left-3 z-10 flex gap-2">
        {(['preset', 'breadthfirst', 'concentric'] as const).map((l) => (
          <button
            key={l}
            onClick={() => runLayout(l)}
            className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors border ${
              layout === l
                ? 'bg-accent/10 border-accent/30 text-accent-hover'
                : 'bg-surface-3/80 border-zinc-700/50 text-zinc-400 hover:text-zinc-300'
            }`}
          >
            {l}
          </button>
        ))}
        <button
          onClick={fitToScreen}
          className="px-3 py-1.5 rounded-lg text-xs font-medium bg-surface-3/80 border border-zinc-700/50 text-zinc-400 hover:text-zinc-300 transition-colors"
        >
          Fit
        </button>
      </div>

      {/* Graph container */}
      <div
        ref={containerRef}
        className="w-full h-[500px] bg-surface rounded-xl border border-zinc-800"
      />

      {/* Node detail panel */}
      {selectedNode && (
        <div className="absolute bottom-3 right-3 w-80 bg-surface-2 border border-zinc-800 rounded-xl p-4 shadow-xl z-10">
          <div className="flex items-center justify-between mb-3">
            <div className="flex items-center gap-2">
              <div
                className="w-3 h-3 rounded-full"
                style={{
                  backgroundColor: CLASS_COLORS[selectedNode.node_type] || '#71717a',
                }}
              />
              <span className="text-sm font-medium text-zinc-200">
                {selectedNode.label}
              </span>
            </div>
            <button
              onClick={() => setSelectedNode(null)}
              className="text-zinc-500 hover:text-zinc-300"
            >
              <X size={14} />
            </button>
          </div>
          <div className="space-y-1.5 text-xs">
            <div className="flex justify-between">
              <span className="text-zinc-500">Type</span>
              <span className="text-zinc-300">{selectedNode.node_type}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-zinc-500">Layer</span>
              <span className="text-zinc-300">{selectedNode.layer}</span>
            </div>
            {selectedNode.metadata.map(([k, v], i) => (
              <div key={i} className="flex justify-between">
                <span className="text-zinc-500 capitalize">{k}</span>
                <span className="text-zinc-300 max-w-[180px] truncate">{v}</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}
