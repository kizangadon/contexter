import { describe, it, expect } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import type { ReactNode } from 'react';
import {
  useAnalyticsOverview,
  useAnalyticsHealth,
  useAnalyticsPerformance,
  useAnalyticsResources,
  useAnalyticsCosts,
  useAnalyticsModelDetail,
  useAnalyticsServices,
} from './useAnalytics';

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

describe('useAnalyticsOverview', () => {
  it('returns analytics overview data from API', async () => {
    const { result } = renderHook(() => useAnalyticsOverview(), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(result.current.data!.system_health).toBe('healthy');
    expect(result.current.data!.uptime_percent).toBeGreaterThan(0);
    expect(result.current.data!.cost_total).toBeGreaterThan(0);
  });

  it('accepts optional timeframe parameter', async () => {
    const { result } = renderHook(() => useAnalyticsOverview('7d'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(result.current.data!.system_health).toBeDefined();
  });

  it('handles loading state correctly', async () => {
    const { result } = renderHook(() => useAnalyticsOverview(), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.isLoading).toBe(false);
  });
});

describe('useAnalyticsHealth', () => {
  it('returns health status from API', async () => {
    const { result } = renderHook(() => useAnalyticsHealth(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(result.current.data!.status).toBe('healthy');
  });

  it('handles loading state', async () => {
    const { result } = renderHook(() => useAnalyticsHealth(), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.isLoading).toBe(false);
  });
});

describe('useAnalyticsPerformance', () => {
  it('returns performance trend data from API', async () => {
    const { result } = renderHook(() => useAnalyticsPerformance(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(Array.isArray(result.current.data)).toBe(true);
    expect(result.current.data!.length).toBeGreaterThan(0);
    expect(result.current.data![0]).toHaveProperty('date');
    expect(result.current.data![0]).toHaveProperty('response_time_ms');
    expect(result.current.data![0]).toHaveProperty('throughput');
  });

  it('accepts optional timeframe parameter', async () => {
    const { result } = renderHook(() => useAnalyticsPerformance('30d'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(Array.isArray(result.current.data)).toBe(true);
  });
});

describe('useAnalyticsResources', () => {
  it('returns resource usage data from API', async () => {
    const { result } = renderHook(() => useAnalyticsResources(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(result.current.data!.cpu_percent).toBeGreaterThanOrEqual(0);
    expect(result.current.data!.memory_percent).toBeGreaterThanOrEqual(0);
    expect(result.current.data!.disk_percent).toBeGreaterThanOrEqual(0);
    expect(result.current.data!.active_connections).toBeGreaterThanOrEqual(0);
  });
});

describe('useAnalyticsCosts', () => {
  it('returns cost breakdown data from API', async () => {
    const { result } = renderHook(() => useAnalyticsCosts(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(result.current.data!.total_cost).toBeGreaterThan(0);
    expect(Array.isArray(result.current.data!.by_model)).toBe(true);
    expect(result.current.data!.by_model.length).toBeGreaterThan(0);
    expect(result.current.data!.by_model[0]).toHaveProperty('model');
  });

  it('accepts optional timeframe parameter', async () => {
    const { result } = renderHook(() => useAnalyticsCosts('7d'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(result.current.data!.total_cost).toBeGreaterThan(0);
  });
});

describe('useAnalyticsModelDetail', () => {
  it('returns model cost detail for valid model id', async () => {
    const { result } = renderHook(() => useAnalyticsModelDetail('gpt-4'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(result.current.data!.model).toBe('gpt-4');
    expect(result.current.data!.total_cost).toBeGreaterThan(0);
    expect(Array.isArray(result.current.data!.daily_breakdown)).toBe(true);
  });

  it('returns error for non-existent model id', async () => {
    const { result } = renderHook(() => useAnalyticsModelDetail('nonexistent-model'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error).toBeDefined();
  });

  it('does not fetch when id is empty', async () => {
    const { result } = renderHook(() => useAnalyticsModelDetail(''), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(false);
    expect(result.current.fetchStatus).toBe('idle');
  });
});

describe('useAnalyticsServices', () => {
  it('returns service status list from API', async () => {
    const { result } = renderHook(() => useAnalyticsServices(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(Array.isArray(result.current.data)).toBe(true);
    expect(result.current.data!.length).toBeGreaterThan(0);
    expect(result.current.data![0]).toHaveProperty('name');
    expect(result.current.data![0]).toHaveProperty('status');
  });
});
