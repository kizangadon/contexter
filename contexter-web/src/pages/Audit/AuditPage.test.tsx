import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, beforeEach } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router';
import { http, HttpResponse } from 'msw';
import type { ReactNode } from 'react';
import { AuditPage } from './AuditPage';
import { server } from '../../../tests/mocks/server';

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={['/audit']}>{children}</MemoryRouter>
      </QueryClientProvider>
    );
  };
}

describe('AuditPage', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  it('renders the page header title', async () => {
    render(<AuditPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /audit/i })).toBeInTheDocument();
    });
  });

  it('renders a table of audit log entries', async () => {
    render(<AuditPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('session.create')).toBeInTheDocument();
      expect(screen.getByText('memory.update')).toBeInTheDocument();
    });
  });

  it('renders column headers for the audit table', async () => {
    render(<AuditPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Timestamp')).toBeInTheDocument();
      expect(screen.getByText('Action')).toBeInTheDocument();
      expect(screen.getByText('User')).toBeInTheDocument();
    });
  });

  it('shows loading skeletons while data is loading', () => {
    render(<AuditPage />, { wrapper: createWrapper() });
    expect(screen.getAllByTestId('skeleton').length).toBeGreaterThan(0);
  });

  it('shows empty state when no audit entries exist', async () => {
    server.use(
      http.get('*/api/v1/audit', () => HttpResponse.json([])),
    );
    render(<AuditPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText(/no audit/i)).toBeInTheDocument();
    });
  });
});
