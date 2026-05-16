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

export function useMetrics(poll = false) {
  return useQuery({
    queryKey: ['metrics'],
    queryFn: api.metrics,
    staleTime: 10_000,
    refetchInterval: poll ? 10_000 : false,
  })
}

export function useSecurity() {
  return useQuery({
    queryKey: ['security'],
    queryFn: api.security,
    staleTime: 30_000,
  })
}

export function useMetricsHistory() {
  return useQuery({
    queryKey: ['metrics-history'],
    queryFn: api.metricsHistory,
    staleTime: 60_000,
  })
}

export function useFindingsHistory() {
  return useQuery({
    queryKey: ['findings-history'],
    queryFn: api.findingsHistory,
    staleTime: 60_000,
  })
}

export function useNetwork(poll = false) {
  return useQuery({
    queryKey: ['network'],
    queryFn: api.network,
    staleTime: 15_000,
    refetchInterval: poll ? 15_000 : false,
  })
}

export function useVisualization() {
  return useQuery({
    queryKey: ['visualization'],
    queryFn: api.visualization,
    staleTime: 30_000,
  })
}
