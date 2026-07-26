import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, beforeEach } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router';
import { http, HttpResponse } from 'msw';
import type { ReactNode } from 'react';
import { EfficiencySessionsPage } from './EfficiencySessionsPage';
import { server } from '../../../tests/mocks/server';

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={['/efficiency/sessions']}>
          {children}
        </MemoryRouter>
      </QueryClientProvider>
    );
  };
}

describe('EfficiencySessionsPage', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  it('renders the page header title', async () => {
    render(<EfficiencySessionsPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Session Activity' })).toBeInTheDocument();
    });
  });

  it('renders stat cards with session metrics', async () => {
    render(<EfficiencySessionsPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Total Sessions')).toBeInTheDocument();
      expect(screen.getByText('Avg Score')).toBeInTheDocument();
      expect(screen.getByText('Total Tokens')).toBeInTheDocument();
    });
  });

  it('renders session data in table', async () => {
    render(<EfficiencySessionsPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('2026-07-20')).toBeInTheDocument();
      expect(screen.getByText('2026-07-26')).toBeInTheDocument();
    });
  });

  it('shows loading skeletons while loading', () => {
    render(<EfficiencySessionsPage />, { wrapper: createWrapper() });
    expect(screen.getAllByTestId('skeleton').length).toBeGreaterThan(0);
  });

  it('shows error state when API fails', async () => {
    server.use(
      http.get('*/api/v1/efficiency/sessions', () => {
        return HttpResponse.json({ detail: 'Error' }, { status: 500 });
      }),
    );
    render(<EfficiencySessionsPage />, { wrapper: createWrapper() });
    expect(await screen.findByRole('button', { name: /retry/i })).toBeInTheDocument();
    expect(screen.getByText('Failed to load session data')).toBeInTheDocument();
  });
});
