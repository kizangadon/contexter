import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, beforeEach, beforeAll } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router';
import { http, HttpResponse } from 'msw';
import type { ReactNode } from 'react';
import { AnalyticsDashboardPage } from './AnalyticsDashboardPage';
import { server } from '../../../tests/mocks/server';

// ResizeObserver polyfill for Recharts ResponsiveContainer in jsdom
beforeAll(() => {
  if (typeof globalThis.ResizeObserver === 'undefined') {
    class MockResizeObserver {
      observe() {}
      unobserve() {}
      disconnect() {}
    }
    globalThis.ResizeObserver =
      MockResizeObserver as unknown as typeof ResizeObserver;
  }
});

/* ─── Wrapper for react-query + router ─────────────────────── */

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: 0,
      },
    },
  });

  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={['/analytics']}>
          {children}
        </MemoryRouter>
      </QueryClientProvider>
    );
  };
}

/* ─── Tests ────────────────────────────────────────────────── */

describe('AnalyticsDashboardPage', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  it('renders the page header title', async () => {
    render(<AnalyticsDashboardPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(
        screen.getByRole('heading', { name: 'Analytics' }),
      ).toBeInTheDocument();
    });
  });

  it('renders timeframe filter with default 30d', async () => {
    render(<AnalyticsDashboardPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      const select = screen.getByRole('combobox', { name: /timeframe/i });
      expect(select).toBeInTheDocument();
      expect((select as HTMLSelectElement).value).toBe('30d');
    });
  });

  it('renders 6 stat cards with overview labels', async () => {
    render(<AnalyticsDashboardPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('System Health')).toBeInTheDocument();
      expect(screen.getByText('Uptime')).toBeInTheDocument();
      expect(screen.getByText('Error Rate')).toBeInTheDocument();
      expect(screen.getByText('Active Sessions')).toBeInTheDocument();
      expect(screen.getByText('Memory Usage')).toBeInTheDocument();
      expect(screen.getByText('Total Cost')).toBeInTheDocument();
    });
  });

  it('renders health section with service statuses', async () => {
    render(<AnalyticsDashboardPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('System Status')).toBeInTheDocument();
      expect(screen.getByText('api')).toBeInTheDocument();
      expect(screen.getByText('database')).toBeInTheDocument();
      expect(screen.getByText('mcp')).toBeInTheDocument();
    });
  });

  it('renders performance section with chart', async () => {
    render(<AnalyticsDashboardPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Performance Trend')).toBeInTheDocument();
    });

    // Recharts renders chart elements — check for a known data point value
    await waitFor(() => {
      // The chart should render; the container should be present
      const chartContainer = document.querySelector(
        '.recharts-responsive-container',
      );
      expect(chartContainer).toBeInTheDocument();
    });
  });

  it('renders resources section with usage cards', async () => {
    render(<AnalyticsDashboardPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Resource Usage')).toBeInTheDocument();
      expect(screen.getByText('CPU')).toBeInTheDocument();
      expect(screen.getByText('Memory')).toBeInTheDocument();
      expect(screen.getByText('Disk')).toBeInTheDocument();
      expect(screen.getByText('Active Connections')).toBeInTheDocument();
    });
  });

  it('renders costs section with cost breakdown', async () => {
    render(<AnalyticsDashboardPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Cost Overview')).toBeInTheDocument();
      expect(screen.getByText('gpt-4')).toBeInTheDocument();
      expect(screen.getByText('gpt-3.5')).toBeInTheDocument();
      expect(screen.getByText('claude-3')).toBeInTheDocument();
    });
  });

  it('shows loading skeletons while data is loading', () => {
    render(<AnalyticsDashboardPage />, { wrapper: createWrapper() });

    // Before data resolves, skeletons should be present
    expect(screen.getAllByTestId('skeleton').length).toBeGreaterThan(0);
  });

  it('shows error state with retry button when API fails', async () => {
    // Override overview handler to return error
    server.use(
      http.get('*/api/v1/analytics/overview', () => {
        return HttpResponse.json(
          { detail: 'Internal server error' },
          { status: 500 },
        );
      }),
    );

    render(<AnalyticsDashboardPage />, { wrapper: createWrapper() });

    // Wait for the retry button to appear
    const retryButton = await screen.findByRole('button', { name: /retry/i });
    expect(retryButton).toBeInTheDocument();

    // Error message should be shown
    expect(
      screen.getByText('Failed to load analytics'),
    ).toBeInTheDocument();
  });
});
