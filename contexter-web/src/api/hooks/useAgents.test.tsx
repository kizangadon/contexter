import { describe, it, expect } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import type { ReactNode } from 'react';
import { useAgents, useAgent } from './useAgents';

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

describe('useAgents', () => {
  it('returns agents list from API', async () => {
    const { result } = renderHook(() => useAgents(), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(Array.isArray(result.current.data)).toBe(true);
    expect(result.current.data!.length).toBeGreaterThan(0);
    expect(result.current.data![0]).toHaveProperty('id');
    expect(result.current.data![0]).toHaveProperty('name');
    expect(result.current.data![0]).toHaveProperty('capabilities');
    expect(result.current.data![0]).toHaveProperty('status');
    expect(result.current.data![0]).toHaveProperty('efficiency_score');
  });

  it('filters by status', async () => {
    const { result } = renderHook(() => useAgents({ status: 'active' }), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    for (const agent of result.current.data ?? []) {
      expect(agent.status).toBe('active');
    }
  });

  it('returns loading state then data', async () => {
    const { result } = renderHook(() => useAgents(), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.isLoading).toBe(false);
    expect(result.current.data).toBeDefined();
  });
});

describe('useAgent', () => {
  it('returns agent detail for valid id', async () => {
    const { result } = renderHook(() => useAgent('agt_000001'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(result.current.data!.id).toBe('agt_000001');
    expect(result.current.data!.recent_sessions).toBeDefined();
    expect(result.current.data!.efficiency_history).toBeDefined();
  });

  it('returns error for non-existent agent id', async () => {
    const { result } = renderHook(() => useAgent('agt_nonexistent'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isError).toBe(true));

    expect(result.current.error).toBeDefined();
  });

  it('does not fetch when id is empty', async () => {
    const { result } = renderHook(() => useAgent(''), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(false);
    expect(result.current.fetchStatus).toBe('idle');
  });
});
