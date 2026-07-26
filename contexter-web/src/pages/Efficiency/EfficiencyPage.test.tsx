import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, beforeEach } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router';
import { http, HttpResponse } from 'msw';
import type { ReactNode } from 'react';
import { EfficiencyPage } from './EfficiencyPage';
import { server } from '../../../tests/mocks/server';

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
        <MemoryRouter initialEntries={['/efficiency']}>
          {children}
        </MemoryRouter>
      </QueryClientProvider>
    );
  };
}

/* ─── Tests ────────────────────────────────────────────────── */

describe('EfficiencyPage', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  it('renders the page header title', async () => {
    render(<EfficiencyPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(
        screen.getByRole('heading', { name: 'Efficiency Mapper' }),
      ).toBeInTheDocument();
    });
  });

  it('renders stat cards with labels from hooks', async () => {
    render(<EfficiencyPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      // Use getAllByText for labels that might appear in multiple places
      expect(screen.getByText('Avg Efficiency')).toBeInTheDocument();
      expect(screen.getAllByText(/Trend/).length).toBeGreaterThanOrEqual(1);
      expect(screen.getByText('Avg Tokens')).toBeInTheDocument();
      expect(screen.getByText('Avg Duration')).toBeInTheDocument();
    });
  });

  it('renders timeframe filter with default selection', async () => {
    render(<EfficiencyPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      const select = screen.getByRole('combobox', { name: /timeframe/i });
      expect(select).toBeInTheDocument();
      expect((select as HTMLSelectElement).value).toBe('30d');
    });
  });

  it('shows loading skeletons while data is loading', () => {
    render(<EfficiencyPage />, { wrapper: createWrapper() });

    // Before data resolves, skeletons should be present
    expect(screen.getAllByTestId('skeleton').length).toBeGreaterThan(0);
  });

  it('shows error state with retry button when API fails', async () => {
    // Override overview handler to return error
    server.use(
      http.get('*/api/v1/efficiency/overview', () => {
        return HttpResponse.json(
          { detail: 'Internal server error' },
          { status: 500 },
        );
      }),
    );

    render(<EfficiencyPage />, { wrapper: createWrapper() });

    // Wait for the retry button to appear
    const retryButton = await screen.findByRole('button', { name: /retry/i });
    expect(retryButton).toBeInTheDocument();

    // Error message should be shown
    expect(
      screen.getByText('Failed to load efficiency data'),
    ).toBeInTheDocument();
  });
});
