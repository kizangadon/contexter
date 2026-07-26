import { http, HttpResponse } from 'msw';
import type { HttpHandler } from 'msw';
import type { OnboardingStatus } from '@/api/types';

let onboardingStatus: OnboardingStatus = {
  current_step: 2,
  total_steps: 4,
  completed: false,
  steps: [
    { id: 'welcome', label: 'Welcome', completed: true },
    { id: 'connect', label: 'Connect Provider', completed: true },
    { id: 'first-session', label: 'First Session', completed: false },
    { id: 'explore', label: 'Explore Dashboard', completed: false },
  ],
};

export const onboardingHandlers: HttpHandler[] = [
  // GET /api/v1/onboarding/status
  http.get('*/api/v1/onboarding/status', () => {
    return HttpResponse.json(onboardingStatus);
  }),

  // POST /api/v1/onboarding/submit — advance onboarding step
  http.post('*/api/v1/onboarding/submit', async ({ request }) => {
    const body = (await request.json()) as { step_id?: string } | undefined;
    const stepId = body?.step_id;

    if (stepId) {
      const step = onboardingStatus.steps.find((s) => s.id === stepId);
      if (step) {
        step.completed = true;
      }
      const nextIndex = onboardingStatus.steps.findIndex((s) => !s.completed);
      onboardingStatus.current_step = nextIndex >= 0 ? nextIndex + 1 : onboardingStatus.total_steps;
      onboardingStatus.completed = nextIndex === -1;
    }

    return HttpResponse.json(onboardingStatus);
  }),
];
