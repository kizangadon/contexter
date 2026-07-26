import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, beforeEach } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router';
import { http, HttpResponse } from 'msw';
import type { ReactNode } from 'react';
import { EfficiencyMemoryPage } from './EfficiencyMemoryPage';
import { server } from '../../../tests/mocks/server';

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={['/efficiency/memory']}>
          {children}
        </MemoryRouter>
      </QueryClientProvider>
    );
  };
}

describe('EfficiencyMemoryPage', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  it('renders the page header title', async () => {
    render(<EfficiencyMemoryPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Memory Usage' })).toBeInTheDocument();
    });
  });

  it('renders stat cards with memory metrics', async () => {
    render(<EfficiencyMemoryPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Total Memories')).toBeInTheDocument();
      expect(screen.getByText('Avg Confidence')).toBeInTheDocument();
      expect(screen.getByText('Memory Types')).toBeInTheDocument();
    });
  });

  it('renders type distribution table', async () => {
    render(<EfficiencyMemoryPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Type Distribution')).toBeInTheDocument();
      expect(screen.getByText('conversation')).toBeInTheDocument();
      expect(screen.getByText('decision')).toBeInTheDocument();
    });
  });

  it('shows loading skeletons while loading', () => {
    render(<EfficiencyMemoryPage />, { wrapper: createWrapper() });
    expect(screen.getAllByTestId('skeleton').length).toBeGreaterThan(0);
  });

  it('shows error state when API fails', async () => {
    server.use(
      http.get('*/api/v1/efficiency/memory', () => {
        return HttpResponse.json({ detail: 'Error' }, { status: 500 });
      }),
    );
    render(<EfficiencyMemoryPage />, { wrapper: createWrapper() });
    expect(await screen.findByRole('button', { name: /retry/i })).toBeInTheDocument();
    expect(screen.getByText('Failed to load memory data')).toBeInTheDocument();
  });
});
