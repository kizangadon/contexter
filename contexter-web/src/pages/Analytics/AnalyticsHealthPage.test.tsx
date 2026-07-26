import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, beforeEach } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router';
import { http, HttpResponse } from 'msw';
import type { ReactNode } from 'react';
import { AnalyticsHealthPage } from './AnalyticsHealthPage';
import { server } from '../../../tests/mocks/server';

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={['/analytics/health']}>
          {children}
        </MemoryRouter>
      </QueryClientProvider>
    );
  };
}

describe('AnalyticsHealthPage', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  it('renders the page header title', async () => {
    render(<AnalyticsHealthPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'System Health' })).toBeInTheDocument();
    });
  });

  it('renders system status badges', async () => {
    render(<AnalyticsHealthPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      // Multiple badges show "healthy" — system status + all services
      const badges = screen.getAllByText('healthy');
      expect(badges.length).toBeGreaterThanOrEqual(1);
    });
  });

  it('renders service status indicators', async () => {
    render(<AnalyticsHealthPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Services')).toBeInTheDocument();
      expect(screen.getByText('api')).toBeInTheDocument();
      expect(screen.getByText('database')).toBeInTheDocument();
      expect(screen.getByText('mcp')).toBeInTheDocument();
    });
  });

  it('shows loading skeletons while data is loading', () => {
    render(<AnalyticsHealthPage />, { wrapper: createWrapper() });
    expect(screen.getAllByTestId('skeleton').length).toBeGreaterThan(0);
  });

  it('shows error state when API fails', async () => {
    server.use(
      http.get('*/api/v1/analytics/health', () => {
        return HttpResponse.json({ detail: 'Error' }, { status: 500 });
      }),
    );
    render(<AnalyticsHealthPage />, { wrapper: createWrapper() });
    expect(await screen.findByRole('button', { name: /retry/i })).toBeInTheDocument();
    expect(screen.getByText('Failed to load health data')).toBeInTheDocument();
  });
});
