import { NavLink } from 'react-router-dom'
import { useAppStore } from '../stores/app'
import { cn } from '../lib/utils'
import {
  LayoutDashboard,
  Network,
  Shield,
  Server,
  Menu,
  ChevronLeft,
} from 'lucide-react'

const navItems = [
  { to: '/', label: 'Dashboard', icon: LayoutDashboard },
  { to: '/topology', label: 'Topology', icon: Network },
  { to: '/security', label: 'Security', icon: Shield },
  { to: '/services', label: 'Services', icon: Server },
]

export function Shell({ children }: { children: React.ReactNode }) {
  const { sidebarOpen, toggleSidebar } = useAppStore()

  return (
    <div className="flex h-screen overflow-hidden">
      {/* Sidebar */}
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

        <div className="p-4 border-t border-zinc-800">
          <div className="flex items-center gap-2 text-xs text-zinc-500">
            <div className="w-2 h-2 rounded-full bg-green" />
            {sidebarOpen && <span>API Connected</span>}
          </div>
        </div>
      </aside>

      {/* Main content */}
      <main className="flex-1 overflow-y-auto">
        <div className="max-w-7xl mx-auto p-6">{children}</div>
      </main>
    </div>
  )
}
