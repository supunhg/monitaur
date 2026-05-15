import { Routes, Route, Navigate } from 'react-router-dom'
import { Shell } from './components/Shell'
import { Dashboard } from './pages/Dashboard'
import { Topology } from './pages/Topology'
import { Security } from './pages/Security'
import { Services } from './pages/Services'

export default function App() {
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
