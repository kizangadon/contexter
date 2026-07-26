import { describe, it, expect } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import type { ReactNode } from 'react';
import { useOnboardingStatus } from './useOnboarding';

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  });

  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
  };
}

describe('useOnboardingStatus', () => {
  it('returns onboarding status from API', async () => {
    const { result } = renderHook(() => useOnboardingStatus(), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(result.current.data!.current_step).toBeGreaterThanOrEqual(0);
    expect(result.current.data!.total_steps).toBeGreaterThan(0);
    expect(result.current.data!.completed).toEqual(false);
    expect(Array.isArray(result.current.data!.steps)).toBe(true);
    expect(result.current.data!.steps.length).toBeGreaterThan(0);
    expect(result.current.data!.steps[0]).toHaveProperty('id');
    expect(result.current.data!.steps[0]).toHaveProperty('label');
    expect(result.current.data!.steps[0]).toHaveProperty('completed');
  });

  it('handles loading state correctly', async () => {
    const { result } = renderHook(() => useOnboardingStatus(), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.isLoading).toBe(false);
  });
});
