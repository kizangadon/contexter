import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, beforeEach, beforeAll } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router';
import { http, HttpResponse } from 'msw';
import type { ReactNode } from 'react';
import { AnalyticsModelsPage } from './AnalyticsModelsPage';
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
        <MemoryRouter initialEntries={['/analytics/models']}>
          {children}
        </MemoryRouter>
      </QueryClientProvider>
    );
  };
}

/* ─── Tests ────────────────────────────────────────────────── */

describe('AnalyticsModelsPage', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  it('renders the page header title', async () => {
    render(<AnalyticsModelsPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(
        screen.getByRole('heading', { name: 'Model Analytics' }),
      ).toBeInTheDocument();
    });
  });

  it('renders breadcrumb trail with Analytics link', async () => {
    render(<AnalyticsModelsPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Analytics')).toBeInTheDocument();
      expect(screen.getByText('Models')).toBeInTheDocument();
    });
  });

  it('renders service status cards with names', async () => {
    render(<AnalyticsModelsPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('API Gateway')).toBeInTheDocument();
      expect(screen.getByText('Database')).toBeInTheDocument();
      expect(screen.getByText('MCP Server')).toBeInTheDocument();
      expect(screen.getByText('Redis Cache')).toBeInTheDocument();
      expect(screen.getByText('Vector Store')).toBeInTheDocument();
    });
  });

  it('shows loading skeletons while data is loading', () => {
    render(<AnalyticsModelsPage />, { wrapper: createWrapper() });

    // Before data resolves, skeletons should be present
    expect(screen.getAllByTestId('skeleton').length).toBeGreaterThan(0);
  });

  it('shows error state with retry button when API fails', async () => {
    // Override services handler to return error
    server.use(
      http.get('*/api/v1/analytics/services', () => {
        return HttpResponse.json(
          { detail: 'Internal server error' },
          { status: 500 },
        );
      }),
    );

    render(<AnalyticsModelsPage />, { wrapper: createWrapper() });

    // Wait for the retry button to appear
    const retryButton = await screen.findByRole('button', { name: /retry/i });
    expect(retryButton).toBeInTheDocument();

    // Error message should be shown
    expect(
      screen.getByText('Failed to load model data'),
    ).toBeInTheDocument();
  });
});
