import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, beforeEach, beforeAll } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router';
import { http, HttpResponse } from 'msw';
import type { ReactNode } from 'react';
import { AnalyticsPerformancePage } from './AnalyticsPerformancePage';
import { server } from '../../../tests/mocks/server';

beforeAll(() => {
  if (typeof globalThis.ResizeObserver === 'undefined') {
    class MockResizeObserver {
      observe() {}
      unobserve() {}
      disconnect() {}
    }
    globalThis.ResizeObserver = MockResizeObserver as unknown as typeof ResizeObserver;
  }
});

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={['/analytics/performance']}>
          {children}
        </MemoryRouter>
      </QueryClientProvider>
    );
  };
}

describe('AnalyticsPerformancePage', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  it('renders the page header title', async () => {
    render(<AnalyticsPerformancePage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Performance Trends' })).toBeInTheDocument();
    });
  });

  it('renders summary stat cards', async () => {
    render(<AnalyticsPerformancePage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Avg Response Time')).toBeInTheDocument();
      expect(screen.getByText('Avg Throughput (req/s)')).toBeInTheDocument();
      expect(screen.getByText('Avg Error Rate')).toBeInTheDocument();
    });
  });

  it('renders chart sections with data', async () => {
    render(<AnalyticsPerformancePage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Response Time Trend')).toBeInTheDocument();
      expect(screen.getByText('Throughput')).toBeInTheDocument();
      expect(screen.getByText('Error Rate')).toBeInTheDocument();
    });
  });

  it('shows loading skeletons while loading', () => {
    render(<AnalyticsPerformancePage />, { wrapper: createWrapper() });
    expect(screen.getAllByTestId('skeleton').length).toBeGreaterThan(0);
  });

  it('shows error state when API fails', async () => {
    server.use(
      http.get('*/api/v1/analytics/performance', () => {
        return HttpResponse.json({ detail: 'Error' }, { status: 500 });
      }),
    );
    render(<AnalyticsPerformancePage />, { wrapper: createWrapper() });
    expect(await screen.findByRole('button', { name: /retry/i })).toBeInTheDocument();
    expect(screen.getByText('Failed to load performance data')).toBeInTheDocument();
  });
});
