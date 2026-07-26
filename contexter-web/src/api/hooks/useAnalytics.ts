import { useQuery } from '@tanstack/react-query';
import type {
  AnalyticsOverview,
  PerformanceTrend,
  ResourceUsage,
  CostBreakdown,
  ModelCostDetail,
  ServiceStatus,
} from '@/api/types';
import { api } from '@/api/client';

function params(timeframe?: string): Record<string, string | undefined> {
  return timeframe ? { timeframe } : {};
}

export function useAnalyticsOverview(timeframe?: string) {
  return useQuery<AnalyticsOverview>({
    queryKey: ['analytics', 'overview', { timeframe }],
    queryFn: () => api.get<AnalyticsOverview>('/analytics/overview', params(timeframe)),
  });
}

export interface HealthStatus {
  status: string;
  uptime_seconds?: number;
  version?: string;
  services?: Record<string, string>;
}

export function useAnalyticsHealth() {
  return useQuery<HealthStatus>({
    queryKey: ['analytics', 'health'],
    queryFn: () => api.get<HealthStatus>('/analytics/health'),
  });
}

export function useAnalyticsPerformance(timeframe?: string) {
  return useQuery<PerformanceTrend[]>({
    queryKey: ['analytics', 'performance', { timeframe }],
    queryFn: () => api.get<PerformanceTrend[]>('/analytics/performance', params(timeframe)),
  });
}

export function useAnalyticsResources() {
  return useQuery<ResourceUsage>({
    queryKey: ['analytics', 'resources'],
    queryFn: () => api.get<ResourceUsage>('/analytics/resources'),
  });
}

export function useAnalyticsCosts(timeframe?: string) {
  return useQuery<CostBreakdown>({
    queryKey: ['analytics', 'costs', { timeframe }],
    queryFn: () => api.get<CostBreakdown>('/analytics/costs', params(timeframe)),
  });
}

export function useAnalyticsModelDetail(id: string) {
  return useQuery<ModelCostDetail>({
    queryKey: ['analytics', 'costs', id],
    queryFn: () => api.get<ModelCostDetail>(`/analytics/costs/${id}`),
    enabled: id.length > 0,
  });
}

export function useAnalyticsServices() {
  return useQuery<ServiceStatus[]>({
    queryKey: ['analytics', 'services'],
    queryFn: () => api.get<ServiceStatus[]>('/analytics/services'),
  });
}
