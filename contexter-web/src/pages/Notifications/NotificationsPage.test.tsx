import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, beforeEach } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router';
import { http, HttpResponse } from 'msw';
import type { ReactNode } from 'react';
import { NotificationsPage } from './NotificationsPage';
import { server } from '../../../tests/mocks/server';

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={['/notifications']}>{children}</MemoryRouter>
      </QueryClientProvider>
    );
  };
}

describe('NotificationsPage', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  it('renders the page header title', async () => {
    render(<NotificationsPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /notifications/i })).toBeInTheDocument();
    });
  });

  it('renders a list of notification cards', async () => {
    render(<NotificationsPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Session Complete')).toBeInTheDocument();
      expect(screen.getByText('High Memory Usage')).toBeInTheDocument();
    });
  });

  it('renders unread count', async () => {
    render(<NotificationsPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText(/3 unread/i)).toBeInTheDocument();
    });
  });

  it('shows a "Mark All Read" button', async () => {
    render(<NotificationsPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /mark all read/i })).toBeInTheDocument();
    });
  });

  it('shows loading skeletons while data is loading', () => {
    render(<NotificationsPage />, { wrapper: createWrapper() });
    expect(screen.getAllByTestId('skeleton').length).toBeGreaterThan(0);
  });

  it('shows empty state when no notifications exist', async () => {
    server.use(
      http.get('*/api/v1/notifications', () => HttpResponse.json([])),
    );
    render(<NotificationsPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText(/no notifications/i)).toBeInTheDocument();
    });
  });
});
