import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, beforeEach } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router';
import { http, HttpResponse } from 'msw';
import type { ReactNode } from 'react';
import { AnalyticsResourcesPage } from './AnalyticsResourcesPage';
import { server } from '../../../tests/mocks/server';

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={['/analytics/resources']}>
          {children}
        </MemoryRouter>
      </QueryClientProvider>
    );
  };
}

describe('AnalyticsResourcesPage', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  it('renders the page header title', async () => {
    render(<AnalyticsResourcesPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Resource Usage' })).toBeInTheDocument();
    });
  });

  it('renders resource cards with labels', async () => {
    render(<AnalyticsResourcesPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      // Labels appear in both cards and table rows — use getAllByText
      expect(screen.getAllByText('CPU').length).toBeGreaterThanOrEqual(1);
      expect(screen.getAllByText('Memory').length).toBeGreaterThanOrEqual(1);
      expect(screen.getAllByText('Disk').length).toBeGreaterThanOrEqual(1);
      expect(screen.getAllByText('Active Connections').length).toBeGreaterThanOrEqual(1);
    });
  });

  it('shows loading skeletons while loading', () => {
    render(<AnalyticsResourcesPage />, { wrapper: createWrapper() });
    expect(screen.getAllByTestId('skeleton').length).toBeGreaterThan(0);
  });

  it('shows error state when API fails', async () => {
    server.use(
      http.get('*/api/v1/analytics/resources', () => {
        return HttpResponse.json({ detail: 'Error' }, { status: 500 });
      }),
    );
    render(<AnalyticsResourcesPage />, { wrapper: createWrapper() });
    expect(await screen.findByRole('button', { name: /retry/i })).toBeInTheDocument();
    expect(screen.getByText('Failed to load resource data')).toBeInTheDocument();
  });
});
