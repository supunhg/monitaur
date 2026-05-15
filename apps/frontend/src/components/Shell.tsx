import { useState, useEffect } from 'react'
import { NavLink } from 'react-router-dom'
import { useAppStore } from '../stores/app'
import { cn } from '../lib/utils'
import { api } from '../lib/api'
import { ErrorBoundary } from './ErrorBoundary'
import {
  LayoutDashboard,
  Network,
  Shield,
  Server,
  Menu,
  ChevronLeft,
  Moon,
  Sun,
} from 'lucide-react'

const navItems = [
  { to: '/', label: 'Dashboard', icon: LayoutDashboard },
  { to: '/topology', label: 'Topology', icon: Network },
  { to: '/security', label: 'Security', icon: Shield },
  { to: '/services', label: 'Services', icon: Server },
]

export function Shell({ children }: { children: React.ReactNode }) {
  const { sidebarOpen, toggleSidebar, darkMode, toggleDarkMode } = useAppStore()
  const [apiConnected, setApiConnected] = useState<boolean | null>(null)

  useEffect(() => {
    let cancelled = false
    const check = async () => {
      try {
        await api.health()
        if (!cancelled) setApiConnected(true)
      } catch {
        if (!cancelled) setApiConnected(false)
      }
    }
    check()
    const interval = setInterval(check, 15_000)
    return () => {
      cancelled = true
      clearInterval(interval)
    }
  }, [])

  return (
    <div className="flex h-screen overflow-hidden">
      <aside
        className={cn(
          'border-r border-zinc-800 bg-surface-2 flex flex-col transition-all duration-200',
          sidebarOpen ? 'w-56' : 'w-14',
        )}
      >
        <div className="flex items-center h-14 px-4 border-b border-zinc-800">
          {sidebarOpen && (
            <span className="text-sm font-semibold tracking-wider text-zinc-100">
              MONITAUR
            </span>
          )}
          <button
            onClick={toggleSidebar}
            className={cn(
              'text-zinc-400 hover:text-zinc-100 transition-colors',
              sidebarOpen ? 'ml-auto' : 'mx-auto',
            )}
          >
            {sidebarOpen ? <ChevronLeft size={18} /> : <Menu size={18} />}
          </button>
        </div>

        <nav className="flex-1 py-4 space-y-1 px-2">
          {navItems.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              end={item.to === '/'}
              className={({ isActive }) =>
                cn(
                  'flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors',
                  isActive
                    ? 'bg-accent/10 text-accent-hover'
                    : 'text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800/50',
                )
              }
            >
              <item.icon size={18} />
              {sidebarOpen && <span>{item.label}</span>}
            </NavLink>
          ))}
        </nav>

        <div className="p-4 border-t border-zinc-800 space-y-3">
          {/* Dark mode toggle */}
          <button
            onClick={toggleDarkMode}
            className="flex items-center gap-3 w-full px-3 py-2 rounded-lg text-sm text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800/50 transition-colors"
          >
            {darkMode ? <Sun size={18} /> : <Moon size={18} />}
            {sidebarOpen && <span>{darkMode ? 'Light mode' : 'Dark mode'}</span>}
          </button>

          {/* API status */}
          <div className="flex items-center gap-2 text-xs text-zinc-500 px-3">
            <div
              className={cn(
                'w-2 h-2 rounded-full',
                apiConnected === null
                  ? 'bg-zinc-500 animate-pulse'
                  : apiConnected
                    ? 'bg-green'
                    : 'bg-red',
              )}
            />
            {sidebarOpen && (
              <span>
                {apiConnected === null
                  ? 'Checking...'
                  : apiConnected
                    ? 'API Connected'
                    : 'API Disconnected'}
              </span>
            )}
          </div>
        </div>
      </aside>

      <main className="flex-1 overflow-y-auto">
        <div className="max-w-7xl mx-auto p-6">
          <ErrorBoundary>{children}</ErrorBoundary>
        </div>
      </main>
    </div>
  )
}
