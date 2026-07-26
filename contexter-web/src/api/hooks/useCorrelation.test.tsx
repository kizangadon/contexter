import { describe, it, expect } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import type { ReactNode } from 'react';
import {
  useCorrelationOverview,
  useCorrelationTimeline,
  useCorrelationCompare,
} from './useCorrelation';

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

describe('useCorrelationOverview', () => {
  it('returns correlation overview data from API', async () => {
    const { result } = renderHook(() => useCorrelationOverview(), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(Array.isArray(result.current.data!.dataset_stats)).toBe(true);
    expect(result.current.data!.dataset_stats.length).toBeGreaterThan(0);
    expect(result.current.data!.dataset_stats[0]).toHaveProperty('variable');
    expect(result.current.data!.dataset_stats[0]).toHaveProperty('mean');
    expect(Array.isArray(result.current.data!.top_correlations)).toBe(true);
    expect(result.current.data!.top_correlations.length).toBeGreaterThan(0);
  });

  it('handles loading state correctly', async () => {
    const { result } = renderHook(() => useCorrelationOverview(), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.isLoading).toBe(false);
  });
});

describe('useCorrelationTimeline', () => {
  it('returns correlation timeline data from API', async () => {
    const { result } = renderHook(() => useCorrelationTimeline(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(Array.isArray(result.current.data)).toBe(true);
    expect(result.current.data!.length).toBeGreaterThan(0);
    expect(result.current.data![0]).toHaveProperty('date');
    expect(Array.isArray(result.current.data![0]!.correlations)).toBe(true);
  });
});

describe('useCorrelationCompare', () => {
  it('returns correlation comparison data from API', async () => {
    const { result } = renderHook(() => useCorrelationCompare(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(Array.isArray(result.current.data!.groups)).toBe(true);
    expect(result.current.data!.groups.length).toBeGreaterThan(0);
    expect(result.current.data!.metric).toBeDefined();
    expect(Array.isArray(result.current.data!.values)).toBe(true);
    expect(result.current.data!.values.length).toBeGreaterThan(0);
    expect(result.current.data!.values[0]).toHaveProperty('group');
    expect(result.current.data!.values[0]).toHaveProperty('mean');
    expect(result.current.data!.test).toBeDefined();
    expect(result.current.data!.test.significant).toBe(true);
  });
});
