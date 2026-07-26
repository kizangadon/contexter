import { describe, it, expect } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import type { ReactNode } from 'react';
import { useSettings } from './useSettings';

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

describe('useSettings', () => {
  it('returns settings section from API', async () => {
    const { result } = renderHook(() => useSettings('general'), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(result.current.data!.key).toBe('general');
    expect(result.current.data!.label).toBe('General Settings');
    expect(result.current.data!.settings).toBeDefined();
  });

  it('returns providers section', async () => {
    const { result } = renderHook(() => useSettings('providers'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(result.current.data!.key).toBe('providers');
    expect(result.current.data!.settings).toHaveProperty('providers');
  });

  it('returns error for non-existent section', async () => {
    const { result } = renderHook(() => useSettings('nonexistent'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error).toBeDefined();
  });

  it('does not fetch when section is empty', async () => {
    const { result } = renderHook(() => useSettings(''), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(false);
    expect(result.current.fetchStatus).toBe('idle');
  });
});
