import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, beforeEach } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router';
import { http, HttpResponse } from 'msw';
import type { ReactNode } from 'react';
import { ExportsPage } from './ExportsPage';
import { server } from '../../../tests/mocks/server';

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={['/exports']}>{children}</MemoryRouter>
      </QueryClientProvider>
    );
  };
}

describe('ExportsPage', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  it('renders the page header title', async () => {
    render(<ExportsPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /exports/i })).toBeInTheDocument();
    });
  });

  it('renders a table of export jobs', async () => {
    render(<ExportsPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('sessions')).toBeInTheDocument();
      expect(screen.getByText('analytics')).toBeInTheDocument();
    });
  });

  it('renders a "New Export" button', async () => {
    render(<ExportsPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /new export/i })).toBeInTheDocument();
    });
  });

  it('shows loading skeletons while data is loading', () => {
    render(<ExportsPage />, { wrapper: createWrapper() });
    expect(screen.getAllByTestId('skeleton').length).toBeGreaterThan(0);
  });

  it('shows empty state when no exports exist', async () => {
    server.use(
      http.get('*/api/v1/exports', () => HttpResponse.json([])),
    );
    render(<ExportsPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText(/no exports/i)).toBeInTheDocument();
    });
  });
});
