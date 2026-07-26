import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import type { BugReport, FeatureRequest, ChangelogEntry } from '@/api/types';
import { api } from '@/api/client';

export function useChangelog() {
  return useQuery<ChangelogEntry[]>({
    queryKey: ['feedback', 'changelog'],
    queryFn: () => api.get<ChangelogEntry[]>('/feedback/changelog'),
  });
}

export function useSubmitBugReport() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: { title: string; description: string; severity: BugReport['severity'] }) =>
      api.post<BugReport>('/feedback/bugs', data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['feedback', 'bugs'] });
    },
  });
}

export function useSubmitSuggestion() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: { title: string; description: string }) =>
      api.post<FeatureRequest>('/feedback/suggestions', data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['feedback', 'suggestions'] });
    },
  });
}
