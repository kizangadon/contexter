import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, beforeEach } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router';
import { http, HttpResponse } from 'msw';
import type { ReactNode } from 'react';
import { EfficiencyAgentsPage } from './EfficiencyAgentsPage';
import { server } from '../../../tests/mocks/server';

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={['/efficiency/agents']}>
          {children}
        </MemoryRouter>
      </QueryClientProvider>
    );
  };
}

describe('EfficiencyAgentsPage', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  it('renders the page header title', async () => {
    render(<EfficiencyAgentsPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Agent Performance' })).toBeInTheDocument();
    });
  });

  it('renders stat cards with agent metrics', async () => {
    render(<EfficiencyAgentsPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Total Agents')).toBeInTheDocument();
      expect(screen.getByText('Avg Efficiency')).toBeInTheDocument();
      expect(screen.getByText('Total Sessions')).toBeInTheDocument();
    });
  });

  it('renders agent data in table', async () => {
    render(<EfficiencyAgentsPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Agent-1')).toBeInTheDocument();
      expect(screen.getByText('Agent-2')).toBeInTheDocument();
      expect(screen.getByText('Agent-3')).toBeInTheDocument();
    });
  });

  it('shows loading skeletons while loading', () => {
    render(<EfficiencyAgentsPage />, { wrapper: createWrapper() });
    expect(screen.getAllByTestId('skeleton').length).toBeGreaterThan(0);
  });

  it('shows error state when API fails', async () => {
    server.use(
      http.get('*/api/v1/efficiency/agents', () => {
        return HttpResponse.json({ detail: 'Error' }, { status: 500 });
      }),
    );
    render(<EfficiencyAgentsPage />, { wrapper: createWrapper() });
    expect(await screen.findByRole('button', { name: /retry/i })).toBeInTheDocument();
    expect(screen.getByText('Failed to load agent data')).toBeInTheDocument();
  });
});
