import { useQuery } from '@tanstack/react-query';
import type {
  EfficiencyOverview,
  EfficiencyDetail,
  AgentPerformance,
  SkillEffectiveness,
  CorrelationMatrix,
  EfficiencyMemory,
  EfficiencyTokens,
} from '@/api/types';
import { api } from '@/api/client';

function effParams(timeframe?: string): Record<string, string | undefined> {
  return timeframe ? { timeframe } : {};
}

export function useEfficiencyOverview(timeframe?: string) {
  return useQuery<EfficiencyOverview>({
    queryKey: ['efficiency', 'overview', { timeframe }],
    queryFn: () => api.get<EfficiencyOverview>('/efficiency/overview', effParams(timeframe)),
  });
}

export function useEfficiencyMemory(timeframe?: string) {
  return useQuery<EfficiencyMemory>({
    queryKey: ['efficiency', 'memory', { timeframe }],
    queryFn: () => api.get<EfficiencyMemory>('/efficiency/memory', effParams(timeframe)),
  });
}

export function useEfficiencySessions(timeframe?: string) {
  return useQuery<EfficiencyDetail[]>({
    queryKey: ['efficiency', 'sessions', { timeframe }],
    queryFn: () => api.get<EfficiencyDetail[]>('/efficiency/sessions', effParams(timeframe)),
  });
}

export function useEfficiencyAgents(timeframe?: string) {
  return useQuery<AgentPerformance[]>({
    queryKey: ['efficiency', 'agents', { timeframe }],
    queryFn: () => api.get<AgentPerformance[]>('/efficiency/agents', effParams(timeframe)),
  });
}

export function useEfficiencySkills(timeframe?: string) {
  return useQuery<SkillEffectiveness[]>({
    queryKey: ['efficiency', 'skills', { timeframe }],
    queryFn: () => api.get<SkillEffectiveness[]>('/efficiency/skills', effParams(timeframe)),
  });
}

export function useEfficiencyTokens(timeframe?: string) {
  return useQuery<EfficiencyTokens>({
    queryKey: ['efficiency', 'tokens', { timeframe }],
    queryFn: () => api.get<EfficiencyTokens>('/efficiency/tokens', effParams(timeframe)),
  });
}

export function useEfficiencyCorrelation(timeframe?: string) {
  return useQuery<CorrelationMatrix>({
    queryKey: ['efficiency', 'correlation', { timeframe }],
    queryFn: () => api.get<CorrelationMatrix>('/efficiency/correlation', effParams(timeframe)),
  });
}
