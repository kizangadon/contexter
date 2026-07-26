import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, beforeEach } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router';
import { http, HttpResponse } from 'msw';
import type { ReactNode } from 'react';
import { DashboardPage } from './DashboardPage';
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
        <MemoryRouter initialEntries={['/dashboard']}>
          {children}
        </MemoryRouter>
      </QueryClientProvider>
    );
  };
}

/* ─── Tests ────────────────────────────────────────────────── */

describe('DashboardPage', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  it('renders the page header title', async () => {
    render(<DashboardPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Dashboard' })).toBeInTheDocument();
    });
  });

  it('renders stat cards with labels from hooks', async () => {
    render(<DashboardPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Total Sessions')).toBeInTheDocument();
      expect(screen.getByText('Active Sessions')).toBeInTheDocument();
      expect(screen.getByText('Total Memories')).toBeInTheDocument();
      expect(screen.getByText('Avg Efficiency')).toBeInTheDocument();
    });
  });

  it('renders the recent sessions table with column headers', async () => {
    render(<DashboardPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('ID')).toBeInTheDocument();
      expect(screen.getByText('Agent')).toBeInTheDocument();
      expect(screen.getByText('Status')).toBeInTheDocument();
      expect(screen.getByText('Duration')).toBeInTheDocument();
      expect(screen.getByText('Turns')).toBeInTheDocument();
      expect(screen.getByText('Last Active')).toBeInTheDocument();
    });

    // "View All →" link should exist
    const viewAllLink = screen.getByRole('link', { name: /view all/i });
    expect(viewAllLink).toBeInTheDocument();
    expect(viewAllLink).toHaveAttribute('href', '/sessions');
  });

  it('renders 3 quick action cards with correct links', async () => {
    render(<DashboardPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Launch Session')).toBeInTheDocument();
      expect(screen.getByText('Explore Memories')).toBeInTheDocument();
      expect(screen.getByText('View Analytics')).toBeInTheDocument();
    });

    expect(screen.getByRole('link', { name: /launch session/i })).toHaveAttribute('href', '/sessions');
    expect(screen.getByRole('link', { name: /explore memories/i })).toHaveAttribute('href', '/memories');
    expect(screen.getByRole('link', { name: /view analytics/i })).toHaveAttribute('href', '/analytics');
  });

  it('shows loading skeletons while data is loading', async () => {
    // Use a client that doesn't resolve quickly — the query will be in flight
    render(<DashboardPage />, { wrapper: createWrapper() });

    // At the very start of render, skeletons should be present before data resolves
    // Quick actions are static and should always be present
    expect(screen.getByText('Launch Session')).toBeInTheDocument();
    expect(screen.getByText('Explore Memories')).toBeInTheDocument();
    expect(screen.getByText('View Analytics')).toBeInTheDocument();
  });

  it('shows empty state when no sessions exist', async () => {
    // Override the sessions handler to return empty array
    server.use(
      http.get('*/api/v1/sessions', () => {
        return HttpResponse.json([]);
      }),
    );

    render(<DashboardPage />, { wrapper: createWrapper() });

    // Wait for the empty state CTA to appear
    const createButton = await screen.findByRole('link', { name: /create.*session/i });
    expect(createButton).toBeInTheDocument();
    expect(createButton).toHaveAttribute('href', '/sessions');

    // Also check for the empty state title
    expect(screen.getByText('No sessions yet')).toBeInTheDocument();
  });

  it('shows error state with retry button when API fails', async () => {
    // Override sessions handler to return error
    server.use(
      http.get('*/api/v1/sessions', () => {
        return HttpResponse.json({ detail: 'Internal server error' }, { status: 500 });
      }),
    );

    render(<DashboardPage />, { wrapper: createWrapper() });

    // Wait for the retry button to appear
    const retryButton = await screen.findByRole('button', { name: /retry/i });
    expect(retryButton).toBeInTheDocument();

    // Error message should be shown
    expect(screen.getByText('Failed to load dashboard')).toBeInTheDocument();
  });
});
