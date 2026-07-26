import { describe, it, expect } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import type { ReactNode } from 'react';
import { useSkills, useSkill } from './useSkills';

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

describe('useSkills', () => {
  it('returns skills list from API', async () => {
    const { result } = renderHook(() => useSkills(), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(Array.isArray(result.current.data)).toBe(true);
    expect(result.current.data!.length).toBeGreaterThan(0);
    expect(result.current.data![0]).toHaveProperty('id');
    expect(result.current.data![0]).toHaveProperty('name');
    expect(result.current.data![0]).toHaveProperty('category');
    expect(result.current.data![0]).toHaveProperty('effectiveness_score');
  });

  it('accepts optional category filter', async () => {
    const { result } = renderHook(() => useSkills({ category: 'code-review' }), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(Array.isArray(result.current.data)).toBe(true);
    for (const skill of result.current.data ?? []) {
      expect(skill.category).toBe('code-review');
    }
  });

  it('handles loading state correctly', async () => {
    const { result } = renderHook(() => useSkills(), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.isLoading).toBe(false);
  });
});

describe('useSkill', () => {
  it('returns skill detail for valid id', async () => {
    const { result } = renderHook(() => useSkill('skl_000001'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(result.current.data!.id).toBe('skl_000001');
    expect(result.current.data!.recent_sessions).toBeDefined();
    expect(result.current.data!.effectiveness_history).toBeDefined();
  });

  it('returns error for non-existent skill id', async () => {
    const { result } = renderHook(() => useSkill('skl_nonexistent'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error).toBeDefined();
  });

  it('does not fetch when id is empty', async () => {
    const { result } = renderHook(() => useSkill(''), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(false);
    expect(result.current.fetchStatus).toBe('idle');
  });
});
