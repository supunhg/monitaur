import { Component, type ReactNode } from 'react'
import { AlertTriangle, RefreshCw } from 'lucide-react'

interface Props {
  children: ReactNode
}

interface State {
  error: Error | null
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props)
    this.state = { error: null }
  }

  static getDerivedStateFromError(error: Error) {
    return { error }
  }

  render() {
    if (this.state.error) {
      return (
        <div className="bg-surface-2 border border-red/20 rounded-xl p-8 text-center space-y-3">
          <AlertTriangle size={40} className="mx-auto text-red" />
          <p className="text-sm text-red">Something went wrong</p>
          <p className="text-xs text-zinc-500">{this.state.error.message}</p>
          <button
            onClick={() => {
              this.setState({ error: null })
              window.location.reload()
            }}
            className="inline-flex items-center gap-2 px-4 py-2 text-sm bg-accent/10 border border-accent/30 text-accent-hover rounded-lg hover:bg-accent/20 transition-colors"
          >
            <RefreshCw size={14} />
            Reload
          </button>
        </div>
      )
    }
    return this.props.children
  }
}
