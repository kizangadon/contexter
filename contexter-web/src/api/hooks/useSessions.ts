import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import type { Session, SessionDetail } from '@/api/types';
import { api } from '@/api/client';

export function useSessions(filter?: { status?: string; project?: string }) {
  return useQuery<Session[]>({
    queryKey: ['sessions', filter],
    queryFn: () => api.get<Session[]>('/sessions', filter as Record<string, string | undefined>),
  });
}

export function useSession(id: string) {
  return useQuery<SessionDetail>({
    queryKey: ['session', id],
    queryFn: () => api.get<SessionDetail>(`/sessions/${id}`),
    enabled: id.length > 0,
  });
}

export function useCreateSession() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: { project?: string; agent?: string }) =>
      api.post<SessionDetail>('/sessions', data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['sessions'] });
    },
  });
}

export function useUpdateSession() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, ...data }: { id: string } & Partial<SessionDetail>) =>
      api.patch<SessionDetail>(`/sessions/${id}`, data),
    onSuccess: (updated) => {
      queryClient.invalidateQueries({ queryKey: ['sessions'] });
      queryClient.setQueryData(['session', updated.id], updated);
    },
  });
}

export function useDeleteSession() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => api.delete<null>(`/sessions/${id}`),
    onMutate: async (id) => {
      // Optimistic removal
      await queryClient.cancelQueries({ queryKey: ['sessions'] });
      const previous = queryClient.getQueryData<Session[]>(['sessions']);
      queryClient.setQueryData<Session[]>(['sessions'], (old) =>
        old?.filter((s) => s.id !== id) ?? [],
      );
      return { previous };
    },
    onError: (_err, _id, context) => {
      // Rollback on error
      if (context?.previous) {
        queryClient.setQueryData(['sessions'], context.previous);
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: ['sessions'] });
    },
  });
}

export function useResumeSession() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => api.post<SessionDetail>(`/sessions/${id}/resume`),
    onSuccess: (updated) => {
      queryClient.invalidateQueries({ queryKey: ['sessions'] });
      queryClient.setQueryData(['session', updated.id], updated);
    },
  });
}
