import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import type { ExportJob } from '@/api/types';
import { api } from '@/api/client';

export function useExports() {
  return useQuery<ExportJob[]>({
    queryKey: ['exports'],
    queryFn: () => api.get<ExportJob[]>('/exports'),
  });
}

export function useSubmitExport() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: { type: ExportJob['type']; format: ExportJob['format'] }) =>
      api.post<ExportJob>('/exports', data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['exports'] });
    },
  });
}
