import { describe, it, expect } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import type { ReactNode } from 'react';
import {
  useEfficiencyOverview,
  useEfficiencySessions,
  useEfficiencyAgents,
  useEfficiencySkills,
  useEfficiencyCorrelation,
} from './useEfficiency';

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

describe('useEfficiencyOverview', () => {
  it('returns efficiency overview data from API', async () => {
    const { result } = renderHook(() => useEfficiencyOverview(), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(result.current.data!.avg_efficiency).toBeGreaterThan(0);
    expect(result.current.data!.session_count).toBeGreaterThan(0);
    expect(result.current.data!.agent_count).toBeGreaterThan(0);
    expect(result.current.data!.skill_count).toBeGreaterThan(0);
  });

  it('accepts optional timeframe parameter', async () => {
    const { result } = renderHook(() => useEfficiencyOverview('7d'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(result.current.data!.avg_efficiency).toBeGreaterThan(0);
  });

  it('handles loading state correctly', async () => {
    const { result } = renderHook(() => useEfficiencyOverview(), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.isLoading).toBe(false);
  });
});

describe('useEfficiencySessions', () => {
  it('returns session efficiency data from API', async () => {
    const { result } = renderHook(() => useEfficiencySessions(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(Array.isArray(result.current.data)).toBe(true);
    expect(result.current.data!.length).toBeGreaterThan(0);
    expect(result.current.data![0]).toHaveProperty('date');
    expect(result.current.data![0]).toHaveProperty('score');
    expect(result.current.data![0]).toHaveProperty('tokens');
  });
});

describe('useEfficiencyAgents', () => {
  it('returns agent efficiency data from API', async () => {
    const { result } = renderHook(() => useEfficiencyAgents(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(Array.isArray(result.current.data)).toBe(true);
    expect(result.current.data!.length).toBeGreaterThan(0);
    expect(result.current.data![0]).toHaveProperty('agent_id');
    expect(result.current.data![0]).toHaveProperty('agent_name');
    expect(result.current.data![0]).toHaveProperty('efficiency_score');
  });
});

describe('useEfficiencySkills', () => {
  it('returns skill effectiveness data from API', async () => {
    const { result } = renderHook(() => useEfficiencySkills(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(Array.isArray(result.current.data)).toBe(true);
    expect(result.current.data!.length).toBeGreaterThan(0);
    expect(result.current.data![0]).toHaveProperty('skill_id');
    expect(result.current.data![0]).toHaveProperty('skill_name');
    expect(result.current.data![0]).toHaveProperty('effectiveness_score');
  });
});

describe('useEfficiencyCorrelation', () => {
  it('returns correlation matrix data from API', async () => {
    const { result } = renderHook(() => useEfficiencyCorrelation(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(Array.isArray(result.current.data!.variables)).toBe(true);
    expect(result.current.data!.variables.length).toBeGreaterThan(0);
    expect(Array.isArray(result.current.data!.correlations)).toBe(true);
  });
});
