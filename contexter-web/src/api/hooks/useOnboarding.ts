import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import type { OnboardingStatus } from '@/api/types';
import { api } from '@/api/client';

export function useOnboardingStatus() {
  return useQuery<OnboardingStatus>({
    queryKey: ['onboarding'],
    queryFn: () => api.get<OnboardingStatus>('/onboarding/status'),
  });
}

export function useSubmitOnboarding() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (stepId: string) =>
      api.post<OnboardingStatus>('/onboarding/submit', { step_id: stepId }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['onboarding'] });
    },
  });
}
