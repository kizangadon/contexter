import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, beforeEach } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router';
import { http, HttpResponse } from 'msw';
import type { ReactNode } from 'react';
import { EfficiencyCorrelationPage } from './EfficiencyCorrelationPage';
import { server } from '../../../tests/mocks/server';

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={['/efficiency/correlation']}>
          {children}
        </MemoryRouter>
      </QueryClientProvider>
    );
  };
}

describe('EfficiencyCorrelationPage', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  it('renders the page header title', async () => {
    render(<EfficiencyCorrelationPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Correlation Matrix' })).toBeInTheDocument();
    });
  });

  it('renders stat cards', async () => {
    render(<EfficiencyCorrelationPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Variables')).toBeInTheDocument();
      expect(screen.getByText('Matrix Size')).toBeInTheDocument();
    });
  });

  it('renders correlation matrix table with variables', async () => {
    render(<EfficiencyCorrelationPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Correlation Coefficients')).toBeInTheDocument();
      // Variable names appear in both th and td — use getAllByText
      expect(screen.getAllByText('efficiency').length).toBeGreaterThanOrEqual(1);
      expect(screen.getAllByText('tokens').length).toBeGreaterThanOrEqual(1);
      expect(screen.getAllByText('sessions').length).toBeGreaterThanOrEqual(1);
      expect(screen.getAllByText('latency').length).toBeGreaterThanOrEqual(1);
    });
  });

  it('shows loading skeletons while loading', () => {
    render(<EfficiencyCorrelationPage />, { wrapper: createWrapper() });
    expect(screen.getAllByTestId('skeleton').length).toBeGreaterThan(0);
  });

  it('shows error state when API fails', async () => {
    server.use(
      http.get('*/api/v1/efficiency/correlation', () => {
        return HttpResponse.json({ detail: 'Error' }, { status: 500 });
      }),
    );
    render(<EfficiencyCorrelationPage />, { wrapper: createWrapper() });
    expect(await screen.findByRole('button', { name: /retry/i })).toBeInTheDocument();
    expect(screen.getByText('Failed to load correlation data')).toBeInTheDocument();
  });
});
