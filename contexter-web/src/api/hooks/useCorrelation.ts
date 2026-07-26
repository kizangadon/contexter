import { useQuery } from '@tanstack/react-query';
import type { CorrelationOverview, CorrelationTimeline, CorrelationCompare } from '@/api/types';
import { api } from '@/api/client';

export function useCorrelationOverview() {
  return useQuery<CorrelationOverview>({
    queryKey: ['correlation', 'overview'],
    queryFn: () => api.get<CorrelationOverview>('/correlation/overview'),
  });
}

export function useCorrelationTimeline() {
  return useQuery<CorrelationTimeline[]>({
    queryKey: ['correlation', 'timeline'],
    queryFn: () => api.get<CorrelationTimeline[]>('/correlation/timeline'),
  });
}

export function useCorrelationCompare() {
  return useQuery<CorrelationCompare>({
    queryKey: ['correlation', 'compare'],
    queryFn: () => api.get<CorrelationCompare>('/correlation/compare'),
  });
}
