import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, beforeEach, beforeAll } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router';
import { http, HttpResponse } from 'msw';
import type { ReactNode } from 'react';
import { AnalyticsCostsPage } from './AnalyticsCostsPage';
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
        <MemoryRouter initialEntries={['/analytics/costs']}>
          {children}
        </MemoryRouter>
      </QueryClientProvider>
    );
  };
}

describe('AnalyticsCostsPage', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  it('renders the page header title', async () => {
    render(<AnalyticsCostsPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Cost Analytics' })).toBeInTheDocument();
    });
  });

  it('renders summary stat cards', async () => {
    render(<AnalyticsCostsPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Total Cost')).toBeInTheDocument();
      expect(screen.getByText('Models Tracked')).toBeInTheDocument();
    });
  });

  it('renders cost by model table', async () => {
    render(<AnalyticsCostsPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Cost by Model')).toBeInTheDocument();
      expect(screen.getByText('gpt-4')).toBeInTheDocument();
      expect(screen.getByText('gpt-3.5')).toBeInTheDocument();
      expect(screen.getByText('claude-3')).toBeInTheDocument();
    });
  });

  it('shows loading skeletons while loading', () => {
    render(<AnalyticsCostsPage />, { wrapper: createWrapper() });
    expect(screen.getAllByTestId('skeleton').length).toBeGreaterThan(0);
  });

  it('shows error state when API fails', async () => {
    server.use(
      http.get('*/api/v1/analytics/costs', () => {
        return HttpResponse.json({ detail: 'Error' }, { status: 500 });
      }),
    );
    render(<AnalyticsCostsPage />, { wrapper: createWrapper() });
    expect(await screen.findByRole('button', { name: /retry/i })).toBeInTheDocument();
    expect(screen.getByText('Failed to load cost data')).toBeInTheDocument();
  });
});
