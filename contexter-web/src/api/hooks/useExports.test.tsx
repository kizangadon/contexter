import { describe, it, expect } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import type { ReactNode } from 'react';
import { useExports } from './useExports';

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

describe('useExports', () => {
  it('returns export history from API', async () => {
    const { result } = renderHook(() => useExports(), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(Array.isArray(result.current.data)).toBe(true);
    expect(result.current.data!.length).toBeGreaterThan(0);
    expect(result.current.data![0]).toHaveProperty('id');
    expect(result.current.data![0]).toHaveProperty('type');
    expect(result.current.data![0]).toHaveProperty('format');
    expect(result.current.data![0]).toHaveProperty('status');
  });

  it('includes exports with different statuses', async () => {
    const { result } = renderHook(() => useExports(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    const statuses = result.current.data!.map((e) => e.status);
    expect(statuses).toContain('completed');
    expect(statuses).toContain('processing');
    expect(statuses).toContain('failed');
  });

  it('handles loading state correctly', async () => {
    const { result } = renderHook(() => useExports(), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.isLoading).toBe(false);
  });
});
