import { useState, useEffect } from 'react'
import { api } from '../lib/api'
import { Shield, Lock, KeyRound, AlertTriangle } from 'lucide-react'

export function Login() {
  const [mode, setMode] = useState<'loading' | 'setup' | 'login'>('loading')
  const [password, setPassword] = useState('')
  const [confirm, setConfirm] = useState('')
  const [error, setError] = useState('')
  const [success, setSuccess] = useState('')

  useEffect(() => {
    api
      .authStatus()
      .then((s) => setMode(s.has_admin ? 'login' : 'setup'))
      .catch(() => setMode('login'))
  }, [])

  const handleSetup = async () => {
    setError('')
    setSuccess('')
    if (password.length < 8) {
      setError('Password must be at least 8 characters')
      return
    }
    if (password !== confirm) {
      setError('Passwords do not match')
      return
    }
    try {
      const res = await api.setup(password)
      api.setToken(res.token)
      setSuccess('Account created! Redirecting...')
      setTimeout(() => window.location.reload(), 1000)
    } catch (e) {
      setError((e as Error).message)
    }
  }

  const handleLogin = async () => {
    setError('')
    setSuccess('')
    if (!password) {
      setError('Password is required')
      return
    }
    try {
      const res = await api.login(password)
      api.setToken(res.token)
      setSuccess('Logged in! Redirecting...')
      setTimeout(() => window.location.reload(), 1000)
    } catch (e) {
      setError((e as Error).message)
    }
  }

  if (mode === 'loading') {
    return (
      <div className="flex items-center justify-center h-screen bg-surface">
        <div className="animate-pulse text-zinc-500">Loading...</div>
      </div>
    )
  }

  return (
    <div className="flex items-center justify-center h-screen bg-surface">
      <div className="w-full max-w-sm mx-auto p-8 space-y-6">
        <div className="text-center space-y-2">
          <div className="flex justify-center">
            <div className="w-12 h-12 rounded-xl bg-accent/10 flex items-center justify-center">
              <Shield size={24} className="text-accent-hover" />
            </div>
          </div>
          <h1 className="text-xl font-semibold text-zinc-100">Monitaur</h1>
          <p className="text-sm text-zinc-500">
            {mode === 'setup'
              ? 'Create an admin account to secure the API'
              : 'Enter your password to access the dashboard'}
          </p>
        </div>

        {error && (
          <div className="flex items-center gap-2 text-sm text-red bg-red/5 border border-red/20 rounded-lg px-4 py-3">
            <AlertTriangle size={14} />
            {error}
          </div>
        )}

        {success && (
          <div className="text-sm text-green bg-green/5 border border-green/20 rounded-lg px-4 py-3 text-center">
            {success}
          </div>
        )}

        <div className="space-y-4">
          {mode === 'setup' && (
            <>
              <div className="space-y-2">
                <label className="text-xs text-zinc-500 uppercase tracking-wider">
                  Password
                </label>
                <div className="relative">
                  <Lock
                    size={16}
                    className="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500"
                  />
                  <input
                    type="password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    onKeyDown={(e) => e.key === 'Enter' && handleSetup()}
                    placeholder="At least 8 characters"
                    className="w-full bg-surface-2 border border-zinc-800 rounded-lg pl-10 pr-4 py-2.5 text-sm text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-accent/50 transition-colors"
                  />
                </div>
              </div>
              <div className="space-y-2">
                <label className="text-xs text-zinc-500 uppercase tracking-wider">
                  Confirm password
                </label>
                <div className="relative">
                  <KeyRound
                    size={16}
                    className="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500"
                  />
                  <input
                    type="password"
                    value={confirm}
                    onChange={(e) => setConfirm(e.target.value)}
                    onKeyDown={(e) => e.key === 'Enter' && handleSetup()}
                    placeholder="Repeat password"
                    className="w-full bg-surface-2 border border-zinc-800 rounded-lg pl-10 pr-4 py-2.5 text-sm text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-accent/50 transition-colors"
                  />
                </div>
              </div>
              <button
                onClick={handleSetup}
                className="w-full px-4 py-2.5 bg-accent hover:bg-accent-hover text-white rounded-lg text-sm font-medium transition-colors"
              >
                Create Account
              </button>
            </>
          )}

          {mode === 'login' && (
            <>
              <div className="space-y-2">
                <label className="text-xs text-zinc-500 uppercase tracking-wider">
                  Password
                </label>
                <div className="relative">
                  <Lock
                    size={16}
                    className="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500"
                  />
                  <input
                    type="password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    onKeyDown={(e) => e.key === 'Enter' && handleLogin()}
                    placeholder="Enter your password"
                    className="w-full bg-surface-2 border border-zinc-800 rounded-lg pl-10 pr-4 py-2.5 text-sm text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-accent/50 transition-colors"
                  />
                </div>
              </div>
              <button
                onClick={handleLogin}
                className="w-full px-4 py-2.5 bg-accent hover:bg-accent-hover text-white rounded-lg text-sm font-medium transition-colors"
              >
                Login
              </button>
            </>
          )}
        </div>

        <p className="text-xs text-center text-zinc-600">
          Data never leaves your machine
        </p>
      </div>
    </div>
  )
}
