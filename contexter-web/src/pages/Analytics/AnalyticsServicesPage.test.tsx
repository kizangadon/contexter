import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, beforeEach } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router';
import { http, HttpResponse } from 'msw';
import type { ReactNode } from 'react';
import { AnalyticsServicesPage } from './AnalyticsServicesPage';
import { server } from '../../../tests/mocks/server';

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={['/analytics/services']}>
          {children}
        </MemoryRouter>
      </QueryClientProvider>
    );
  };
}

describe('AnalyticsServicesPage', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  it('renders the page header title', async () => {
    render(<AnalyticsServicesPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Service Status' })).toBeInTheDocument();
    });
  });

  it('renders service cards with statuses', async () => {
    render(<AnalyticsServicesPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('API Gateway')).toBeInTheDocument();
      expect(screen.getByText('Database')).toBeInTheDocument();
      expect(screen.getByText('MCP Server')).toBeInTheDocument();
    });
  });

  it('renders summary stat cards', async () => {
    render(<AnalyticsServicesPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Total Services')).toBeInTheDocument();
      expect(screen.getByText('Healthy')).toBeInTheDocument();
      expect(screen.getByText('Degraded / Down')).toBeInTheDocument();
    });
  });

  it('shows loading skeletons while loading', () => {
    render(<AnalyticsServicesPage />, { wrapper: createWrapper() });
    expect(screen.getAllByTestId('skeleton').length).toBeGreaterThan(0);
  });

  it('shows error state when API fails', async () => {
    server.use(
      http.get('*/api/v1/analytics/services', () => {
        return HttpResponse.json({ detail: 'Error' }, { status: 500 });
      }),
    );
    render(<AnalyticsServicesPage />, { wrapper: createWrapper() });
    expect(await screen.findByRole('button', { name: /retry/i })).toBeInTheDocument();
    expect(screen.getByText('Failed to load service data')).toBeInTheDocument();
  });
});
