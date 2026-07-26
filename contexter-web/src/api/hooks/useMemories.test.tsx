import { describe, it, expect } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import type { ReactNode } from 'react';
import { useMemories, useMemory } from './useMemories';

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

describe('useMemories', () => {
  it('returns memories list from API', async () => {
    const { result } = renderHook(() => useMemories(), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(Array.isArray(result.current.data)).toBe(true);
    expect(result.current.data!.length).toBeGreaterThan(0);
    expect(result.current.data![0]).toHaveProperty('id');
    expect(result.current.data![0]).toHaveProperty('memory_type');
    expect(result.current.data![0]).toHaveProperty('confidence');
  });

  it('filters by memory_type', async () => {
    const { result } = renderHook(() => useMemories({ memory_type: 'conversation' }), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    for (const memory of result.current.data ?? []) {
      expect(memory.memory_type).toBe('conversation');
    }
  });

  it('returns loading state then data', async () => {
    const { result } = renderHook(() => useMemories(), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.isLoading).toBe(false);
    expect(result.current.data).toBeDefined();
  });
});

describe('useMemory', () => {
  it('returns memory detail for valid id', async () => {
    const { result } = renderHook(() => useMemory('mem_000001'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(result.current.data!.id).toBe('mem_000001');
    expect(result.current.data!.versions).toBeDefined();
  });

  it('returns error for non-existent memory id', async () => {
    const { result } = renderHook(() => useMemory('mem_nonexistent'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isError).toBe(true));

    expect(result.current.error).toBeDefined();
  });

  it('does not fetch when id is empty', async () => {
    const { result } = renderHook(() => useMemory(''), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(false);
    expect(result.current.fetchStatus).toBe('idle');
  });
});
