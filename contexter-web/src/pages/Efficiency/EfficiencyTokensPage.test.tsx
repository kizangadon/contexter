import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, beforeEach, beforeAll } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router';
import { http, HttpResponse } from 'msw';
import type { ReactNode } from 'react';
import { EfficiencyTokensPage } from './EfficiencyTokensPage';
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
        <MemoryRouter initialEntries={['/efficiency/tokens']}>
          {children}
        </MemoryRouter>
      </QueryClientProvider>
    );
  };
}

describe('EfficiencyTokensPage', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  it('renders the page header title', async () => {
    render(<EfficiencyTokensPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Token Usage' })).toBeInTheDocument();
    });
  });

  it('renders stat cards with token metrics', async () => {
    render(<EfficiencyTokensPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Total Tokens')).toBeInTheDocument();
      expect(screen.getByText('Avg per Session')).toBeInTheDocument();
      expect(screen.getByText('Models Used')).toBeInTheDocument();
    });
  });

  it('renders by-model breakdown table', async () => {
    render(<EfficiencyTokensPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Tokens by Model')).toBeInTheDocument();
      expect(screen.getByText('gpt-4')).toBeInTheDocument();
      expect(screen.getByText('gpt-3.5')).toBeInTheDocument();
    });
  });

  it('shows loading skeletons while loading', () => {
    render(<EfficiencyTokensPage />, { wrapper: createWrapper() });
    expect(screen.getAllByTestId('skeleton').length).toBeGreaterThan(0);
  });

  it('shows error state when API fails', async () => {
    server.use(
      http.get('*/api/v1/efficiency/tokens', () => {
        return HttpResponse.json({ detail: 'Error' }, { status: 500 });
      }),
    );
    render(<EfficiencyTokensPage />, { wrapper: createWrapper() });
    expect(await screen.findByRole('button', { name: /retry/i })).toBeInTheDocument();
    expect(screen.getByText('Failed to load token data')).toBeInTheDocument();
  });
});
