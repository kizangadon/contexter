import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, beforeEach } from 'vitest';
import { MemoryRouter, Route, Routes } from 'react-router';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { http, HttpResponse } from 'msw';
import { SessionManagerPage } from './SessionManagerPage';
import { server } from '../../../tests/mocks/server';
import type { Session } from '@/api/types';
import { buildSession } from '../../../tests/mocks/factories/sessionFactory';

function createWrapper(initialRoute = '/sessions') {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false, gcTime: 0 },
    },
  });

  return function Wrapper() {
    return (
      <MemoryRouter initialEntries={[initialRoute]}>
        <QueryClientProvider client={queryClient}>
          <Routes>
            <Route path="/sessions" element={<SessionManagerPage />} />
            <Route path="/sessions/:id" element={<div data-testid="session-detail-page">Detail</div>} />
          </Routes>
        </QueryClientProvider>
      </MemoryRouter>
    );
  };
}

describe('SessionManagerPage', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  it('renders the page title', async () => {
    render(<SessionManagerPage />, { wrapper: createWrapper() });
    expect(screen.getByRole('heading', { name: 'Sessions' })).toBeInTheDocument();
  });

  it('renders New Session button', async () => {
    render(<SessionManagerPage />, { wrapper: createWrapper() });
    expect(screen.getByRole('link', { name: /new session/i })).toBeInTheDocument();
  });

  it('renders FilterBar with status options', async () => {
    render(<SessionManagerPage />, { wrapper: createWrapper() });
    // "Status" appears in FilterBar label AND DataTable column header
    const statusLabels = screen.getAllByText('Status');
    expect(statusLabels.length).toBeGreaterThanOrEqual(1);
    expect(screen.getByLabelText('Status')).toBeInTheDocument();
  });

  it('shows loading skeletons initially', () => {
    render(<SessionManagerPage />, { wrapper: createWrapper() });
    const skeletons = screen.getAllByTestId('skeleton');
    expect(skeletons.length).toBeGreaterThan(0);
  });

  it('renders session rows after loading', async () => {
    render(<SessionManagerPage />, { wrapper: createWrapper() });

    // Wait for data to load — MSW returns 3 session
    await waitFor(() => {
      expect(screen.getByText('ses_000001')).toBeInTheDocument();
    });

    expect(screen.getByText('ses_000002')).toBeInTheDocument();
    expect(screen.getByText('ses_000003')).toBeInTheDocument();
  });

  it('renders all column headers', async () => {
    render(<SessionManagerPage />, { wrapper: createWrapper() });

    await screen.findByText('ses_000001');

    expect(screen.getByText('ID')).toBeInTheDocument();
    expect(screen.getByText('Agent')).toBeInTheDocument();
    // "Status" appears in both FilterBar and DataTable
    const statusHeaders = screen.getAllByText('Status');
    expect(statusHeaders.length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText('Duration')).toBeInTheDocument();
    expect(screen.getByText('Turns')).toBeInTheDocument();
    expect(screen.getByText('Last Active')).toBeInTheDocument();
  });

  it('navigates to session detail on row click', async () => {
    render(<SessionManagerPage />, { wrapper: createWrapper() });

    await screen.findByText('ses_000001');

    // Click on the first row
    const rows = screen.getAllByRole('row');
    // DataTable renders header row + data rows. Click on first data row.
    const firstDataRow = rows[1]!;
    await userEvent.click(firstDataRow);

    await waitFor(() => {
      expect(screen.getByTestId('session-detail-page')).toBeInTheDocument();
    });
  });

  it('filters sessions by status', async () => {
    const user = userEvent.setup();
    render(<SessionManagerPage />, { wrapper: createWrapper() });

    await screen.findByText('ses_000001');

    // Change status filter to "done" — MSW handler filters by status
    const statusSelect = screen.getByLabelText('Status');
    await user.selectOptions(statusSelect, 'done');

    // The list should update — MSW filters based on status param
    // All seeded sessions are 'active', so after filtering to 'done' none should show
    await waitFor(() => {
      expect(screen.getByText('No sessions yet')).toBeInTheDocument();
    });
  });

  it('shows empty state when no sessions exist', async () => {
    server.use(
      http.get('*/api/v1/sessions', () => {
        return HttpResponse.json([]);
      }),
    );

    render(<SessionManagerPage />, { wrapper: createWrapper() });

    expect(await screen.findByText('No sessions yet')).toBeInTheDocument();
    expect(screen.getByText('Create your first session to get started with Contexter.')).toBeInTheDocument();
  });

  it('shows pagination for many sessions', async () => {
    // Generate >10 sessions to trigger pagination
    const manySessions: Session[] = Array.from({ length: 12 }, (_, i) =>
      buildSession({ id: `ses_${String(i + 1).padStart(6, '0')}`, status: 'active' }),
    );

    server.use(
      http.get('*/api/v1/sessions', () => {
        return HttpResponse.json(manySessions);
      }),
    );

    render(<SessionManagerPage />, { wrapper: createWrapper() });

    await screen.findByText('ses_000001');

    // Pagination controls should appear
    expect(screen.getByText('Page 1 of 2')).toBeInTheDocument();
    expect(screen.getByText('Next')).toBeInTheDocument();
  });

  it('shows error state with retry', async () => {
    server.use(
      http.get('*/api/v1/sessions', () => {
        return HttpResponse.json({ detail: 'Server error' }, { status: 500 });
      }),
    );

    render(<SessionManagerPage />, { wrapper: createWrapper() });

    expect(await screen.findByText('Failed to load sessions')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /retry/i })).toBeInTheDocument();
  });

  it('sorts by duration when clicking Duration column header', async () => {
    const sessions: Session[] = [
      buildSession({ id: 'ses_000001', duration_minutes: 10 }),
      buildSession({ id: 'ses_000002', duration_minutes: 30 }),
      buildSession({ id: 'ses_000003', duration_minutes: 20 }),
    ];

    server.use(
      http.get('*/api/v1/sessions', () => {
        return HttpResponse.json(sessions);
      }),
    );

    render(<SessionManagerPage />, { wrapper: createWrapper() });

    await screen.findByText('ses_000001');

    // Click Duration header to sort
    const durationHeader = screen.getByText('Duration');
    await userEvent.click(durationHeader);

    // After sorting ascending, ses_000001 (10m) should be first
    const rows = screen.getAllByRole('row');
    expect(rows[1]).toHaveTextContent('ses_000001');
  });
});
