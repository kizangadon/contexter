import { describe, it, expect } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import type { ReactNode } from 'react';
import { useChangelog } from './useFeedback';

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

describe('useChangelog', () => {
  it('returns changelog entries from API', async () => {
    const { result } = renderHook(() => useChangelog(), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(Array.isArray(result.current.data)).toBe(true);
    expect(result.current.data!.length).toBeGreaterThan(0);
    expect(result.current.data![0]).toHaveProperty('version');
    expect(result.current.data![0]).toHaveProperty('date');
    expect(Array.isArray(result.current.data![0]!.changes)).toBe(true);
    expect(result.current.data![0]!.changes.length).toBeGreaterThan(0);
    expect(result.current.data![0]!.changes[0]!).toHaveProperty('type');
    expect(result.current.data![0]!.changes[0]!).toHaveProperty('description');
  });

  it('contains the latest version entry', async () => {
    const { result } = renderHook(() => useChangelog(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data![0]!.version).toBe('1.1.0');
  });

  it('handles loading state correctly', async () => {
    const { result } = renderHook(() => useChangelog(), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.isLoading).toBe(false);
  });
});
