import { useQuery } from '@tanstack/react-query'
import { api } from '../lib/api'

export function useScan() {
  return useQuery({
    queryKey: ['scan'],
    queryFn: api.scan,
    staleTime: 30_000,
  })
}

export function useServices() {
  return useQuery({
    queryKey: ['services'],
    queryFn: api.services,
    staleTime: 30_000,
  })
}

export function useService(id: string) {
  return useQuery({
    queryKey: ['service', id],
    queryFn: () => api.service(id),
    enabled: !!id,
  })
}

export function useMetrics() {
  return useQuery({
    queryKey: ['metrics'],
    queryFn: api.metrics,
    staleTime: 15_000,
  })
}

export function useSecurity() {
  return useQuery({
    queryKey: ['security'],
    queryFn: api.security,
    staleTime: 30_000,
  })
}

export function useNetwork() {
  return useQuery({
    queryKey: ['network'],
    queryFn: api.network,
    staleTime: 15_000,
  })
}

export function useVisualization() {
  return useQuery({
    queryKey: ['visualization'],
    queryFn: api.visualization,
    staleTime: 30_000,
  })
}
