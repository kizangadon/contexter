import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, beforeEach } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router';
import { http, HttpResponse } from 'msw';
import type { ReactNode } from 'react';
import { CorrelationPage } from './CorrelationPage';
import { server } from '../../../tests/mocks/server';

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={['/correlation']}>{children}</MemoryRouter>
      </QueryClientProvider>
    );
  };
}

describe('CorrelationPage', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  it('renders the page header title', async () => {
    render(<CorrelationPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /correlation/i })).toBeInTheDocument();
    });
  });

  it('renders overview stat cards', async () => {
    render(<CorrelationPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Top Correlations')).toBeInTheDocument();
    });
  });

  it('shows loading skeletons while data is loading', () => {
    render(<CorrelationPage />, { wrapper: createWrapper() });
    expect(screen.getAllByTestId('skeleton').length).toBeGreaterThan(0);
  });

  it('shows error state with retry button when API fails', async () => {
    server.use(
      http.get('*/api/v1/correlation/overview', () => {
        return HttpResponse.json({ detail: 'Error' }, { status: 500 });
      }),
    );
    render(<CorrelationPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /retry/i })).toBeInTheDocument();
    });
  });
});
