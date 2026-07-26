import { describe, it, expect } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import type { ReactNode } from 'react';
import { useAudit } from './useAudit';

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

describe('useAudit', () => {
  it('returns audit entries from API', async () => {
    const { result } = renderHook(() => useAudit(), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(Array.isArray(result.current.data)).toBe(true);
    expect(result.current.data!.length).toBeGreaterThan(0);
    expect(result.current.data![0]).toHaveProperty('id');
    expect(result.current.data![0]).toHaveProperty('action');
    expect(result.current.data![0]).toHaveProperty('entity_type');
    expect(result.current.data![0]).toHaveProperty('entity_id');
    expect(result.current.data![0]).toHaveProperty('changes');
    expect(result.current.data![0]).toHaveProperty('performed_by');
  });

  it('includes entries with different action types', async () => {
    const { result } = renderHook(() => useAudit(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    const actions = result.current.data!.map((e) => e.action);
    expect(actions).toContain('session.create');
    expect(actions).toContain('memory.update');
  });

  it('handles loading state correctly', async () => {
    const { result } = renderHook(() => useAudit(), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.isLoading).toBe(false);
  });
});
