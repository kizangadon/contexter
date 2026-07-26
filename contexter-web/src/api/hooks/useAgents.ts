import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import type { Agent, AgentDetail } from '@/api/types';
import { api } from '@/api/client';

export function useAgents(filter?: { status?: string }) {
  return useQuery<Agent[]>({
    queryKey: ['agents', filter],
    queryFn: () => api.get<Agent[]>('/agents', filter as Record<string, string | undefined>),
  });
}

export function useAgent(id: string) {
  return useQuery<AgentDetail>({
    queryKey: ['agent', id],
    queryFn: () => api.get<AgentDetail>(`/agents/${id}`),
    enabled: id.length > 0,
  });
}

export function useCreateAgent() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: { name: string; capabilities?: string[] }) =>
      api.post<AgentDetail>('/agents', data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['agents'] });
    },
  });
}
