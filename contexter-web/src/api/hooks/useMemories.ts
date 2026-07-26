import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import type { Memory, MemoryDetail, MemoryVersion } from '@/api/types';
import { api } from '@/api/client';

export function useMemories(filter?: { memory_type?: string; tags?: string[] }) {
  return useQuery<Memory[]>({
    queryKey: ['memories', filter],
    queryFn: () =>
      api.get<Memory[]>('/memories', {
        memory_type: filter?.memory_type,
        tags: filter?.tags?.join(','),
      }),
  });
}

export function useMemory(id: string) {
  return useQuery<MemoryDetail>({
    queryKey: ['memory', id],
    queryFn: () => api.get<MemoryDetail>(`/memories/${id}`),
    enabled: id.length > 0,
  });
}

export function useMemoryVersions(id: string) {
  return useQuery<MemoryVersion[]>({
    queryKey: ['memory', id, 'versions'],
    queryFn: () => api.get<MemoryVersion[]>(`/memories/${id}/versions`),
    enabled: id.length > 0,
  });
}

export function useMemorySearch(query: string) {
  return useQuery<Memory[]>({
    queryKey: ['memories', 'search', query],
    queryFn: () => api.get<Memory[]>('/memories/search', { q: query }),
    enabled: query.length >= 2,
  });
}

export function useCreateMemory() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: { content: string; memory_type?: Memory['memory_type']; tags?: string[] }) =>
      api.post<MemoryDetail>('/memories', data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['memories'] });
    },
  });
}

export function useUpdateMemory() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, ...data }: { id: string } & Partial<MemoryDetail>) =>
      api.patch<MemoryDetail>(`/memories/${id}`, data),
    onSuccess: (updated) => {
      queryClient.invalidateQueries({ queryKey: ['memories'] });
      queryClient.setQueryData(['memory', updated.id], updated);
    },
  });
}

export function useDeleteMemory() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => api.delete<null>(`/memories/${id}`),
    onMutate: async (id) => {
      await queryClient.cancelQueries({ queryKey: ['memories'] });
      const previous = queryClient.getQueryData<Memory[]>(['memories']);
      queryClient.setQueryData<Memory[]>(['memories'], (old) =>
        old?.filter((m) => m.id !== id) ?? [],
      );
      return { previous };
    },
    onError: (_err, _id, context) => {
      if (context?.previous) {
        queryClient.setQueryData(['memories'], context.previous);
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: ['memories'] });
    },
  });
}
