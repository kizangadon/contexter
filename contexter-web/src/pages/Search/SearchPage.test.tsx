import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, beforeEach } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router';
import { http } from 'msw';
import type { ReactNode } from 'react';
import { SearchPage } from './SearchPage';
import { server } from '../../../tests/mocks/server';

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={['/search']}>{children}</MemoryRouter>
      </QueryClientProvider>
    );
  };
}

describe('SearchPage', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  it('renders the page header title', async () => {
    render(<SearchPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Search' })).toBeInTheDocument();
    });
  });

  it('renders a search input field', async () => {
    render(<SearchPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByPlaceholderText(/search/i)).toBeInTheDocument();
    });
  });

  it('shows results after typing a query with >= 2 characters', async () => {
    render(<SearchPage />, { wrapper: createWrapper() });
    const input = screen.getByPlaceholderText(/search/i);
    await userEvent.type(input, 'code');
    await waitFor(() => {
      expect(screen.getByText('Code Review Session')).toBeInTheDocument();
    });
  });

  it('shows loading skeleton while searching', async () => {
    server.use(
      http.get('*/api/v1/search', () => {
        return new Promise(() => {}); // Never resolves
      }),
    );
    render(<SearchPage />, { wrapper: createWrapper() });
    const input = screen.getByPlaceholderText(/search/i);
    await userEvent.type(input, 'code');
    await waitFor(() => {
      expect(screen.getAllByTestId('skeleton').length).toBeGreaterThan(0);
    });
  });

  it('shows empty state when no results match', async () => {
    render(<SearchPage />, { wrapper: createWrapper() });
    const input = screen.getByPlaceholderText(/search/i);
    await userEvent.type(input, 'zzzzzz');
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /no results/i })).toBeInTheDocument();
    });
  });
});
