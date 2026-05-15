import { useEffect, useState } from 'react'
import { Routes, Route, Navigate, useNavigate } from 'react-router-dom'
import { Shell } from './components/Shell'
import { Dashboard } from './pages/Dashboard'
import { Topology } from './pages/Topology'
import { Security } from './pages/Security'
import { Services } from './pages/Services'
import { Login } from './pages/Login'
import { api } from './lib/api'

function ProtectedApp() {
  return (
    <Shell>
      <Routes>
        <Route path="/" element={<Dashboard />} />
        <Route path="/topology" element={<Topology />} />
        <Route path="/security" element={<Security />} />
        <Route path="/services" element={<Services />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </Shell>
  )
}

export default function App() {
  const [authState, setAuthState] = useState<'loading' | 'login' | 'app'>('loading')
  const navigate = useNavigate()

  useEffect(() => {
    const handleUnauthorized = () => {
      setAuthState('login')
      navigate('/login', { replace: true })
    }
    window.addEventListener('monitaur:unauthorized', handleUnauthorized)
    return () => window.removeEventListener('monitaur:unauthorized', handleUnauthorized)
  }, [navigate])

  useEffect(() => {
    api
      .authStatus()
      .then((s) => {
        if (s.auth_enabled && !api.getToken()) {
          setAuthState('login')
        } else if (s.auth_enabled && api.getToken()) {
          // Validate the stored token
          api.health().then(
            () => setAuthState('app'),
            () => setAuthState('login'),
          )
        } else {
          setAuthState('app')
        }
      })
      .catch(() => setAuthState('app'))
  }, [])

  if (authState === 'loading') {
    return (
      <div className="flex items-center justify-center h-screen bg-surface">
        <div className="animate-pulse text-zinc-500">Loading...</div>
      </div>
    )
  }

  if (authState === 'login') {
    return (
      <Routes>
        <Route path="*" element={<Login />} />
      </Routes>
    )
  }

  return <ProtectedApp />
}
