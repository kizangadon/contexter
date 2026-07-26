import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import type { SettingsSection } from '@/api/types';
import { api } from '@/api/client';

export function useSettings(section: string) {
  return useQuery<SettingsSection>({
    queryKey: ['settings', section],
    queryFn: () => api.get<SettingsSection>(`/settings/${section}`),
    enabled: section.length > 0,
  });
}

export function useUpdateSettings() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ section, data }: { section: string; data: Record<string, unknown> }) =>
      api.put<SettingsSection>(`/settings/${section}`, data),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ['settings', variables.section] });
    },
  });
}
