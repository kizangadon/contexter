import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, beforeEach, beforeAll } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter, Route, Routes } from 'react-router';
import { http, HttpResponse } from 'msw';
import type { ReactNode } from 'react';
import { AnalyticsModelDetailPage } from './AnalyticsModelDetailPage';
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

function createWrapper(route = '/analytics/costs/models/gpt-4') {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={[route]}>
          <Routes>
            <Route path="/analytics/costs/models/:id" element={children} />
          </Routes>
        </MemoryRouter>
      </QueryClientProvider>
    );
  };
}

describe('AnalyticsModelDetailPage', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  it('renders the model name as page title', async () => {
    render(<AnalyticsModelDetailPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'gpt-4' })).toBeInTheDocument();
    });
  });

  it('renders stat cards with model metrics', async () => {
    render(<AnalyticsModelDetailPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Total Cost')).toBeInTheDocument();
      expect(screen.getByText('Total Tokens')).toBeInTheDocument();
      expect(screen.getByText('Input Tokens')).toBeInTheDocument();
      expect(screen.getByText('Cost per Token')).toBeInTheDocument();
    });
  });

  it('shows loading skeletons while loading', () => {
    render(<AnalyticsModelDetailPage />, { wrapper: createWrapper() });
    expect(screen.getAllByTestId('skeleton').length).toBeGreaterThan(0);
  });

  it('shows error state when API fails', async () => {
    server.use(
      http.get('*/api/v1/analytics/costs/:model', () => {
        return HttpResponse.json({ detail: 'Error' }, { status: 500 });
      }),
    );
    render(<AnalyticsModelDetailPage />, { wrapper: createWrapper() });
    expect(await screen.findByRole('button', { name: /retry/i })).toBeInTheDocument();
    expect(screen.getByText('Failed to load model data')).toBeInTheDocument();
  });

  it('shows no-model state when id is missing', () => {
    // Render without a matching route so useParams returns undefined
    const noRouteQueryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0 } },
    });
    render(<AnalyticsModelDetailPage />, {
      wrapper: function NoRouteWrapper({ children }: { children: ReactNode }) {
        return (
          <QueryClientProvider client={noRouteQueryClient}>
            <MemoryRouter initialEntries={['/']}>
              {children}
            </MemoryRouter>
          </QueryClientProvider>
        );
      },
    });
    expect(screen.getByText('No model specified')).toBeInTheDocument();
  });
});
