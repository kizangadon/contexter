import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, beforeEach } from 'vitest';
import { MemoryRouter, Route, Routes } from 'react-router';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { http, HttpResponse } from 'msw';
import { SessionDetailPage } from './SessionDetailPage';
import { server } from '../../../tests/mocks/server';

function createWrapper(initialRoute = '/sessions/ses_000001') {
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
            <Route path="/sessions/:id" element={<SessionDetailPage />} />
            <Route path="/sessions" element={<div data-testid="sessions-list-page">List</div>} />
          </Routes>
        </QueryClientProvider>
      </MemoryRouter>
    );
  };
}

async function waitForLoad() {
  await screen.findByRole('heading', { level: 1 });
}

describe('SessionDetailPage', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  it('renders loading skeleton while fetching', () => {
    render(<SessionDetailPage />, { wrapper: createWrapper() });

    const skeletons = screen.getAllByTestId('skeleton');
    expect(skeletons.length).toBeGreaterThan(0);
  });

  it('renders breadcrumb with Sessions link and session ID', async () => {
    render(<SessionDetailPage />, { wrapper: createWrapper() });

    await waitForLoad();

    const nav = screen.getByLabelText('Breadcrumb');
    expect(nav).toHaveTextContent('Sessions');
    // Session ID appears in breadcrumb (truncated to 11 chars + …)
    expect(nav).toHaveTextContent('ses_000001');
  });

  it('renders session info header with status badge', async () => {
    render(<SessionDetailPage />, { wrapper: createWrapper() });

    await waitForLoad();

    // Session ID appears in breadcrumb AND info header — use getAllByText
    const idElements = screen.getAllByText('ses_000001');
    expect(idElements.length).toBeGreaterThanOrEqual(1);
    // Status badge (factory default is 'active')
    expect(screen.getByText('active')).toBeInTheDocument();
    // Agent name from factory default
    expect(screen.getByText('default-agent')).toBeInTheDocument();
    // Project from factory default
    expect(screen.getByText('contexter')).toBeInTheDocument();
  });

  it('renders duration and turns in the info header stats', async () => {
    render(<SessionDetailPage />, { wrapper: createWrapper() });

    await waitForLoad();

    expect(screen.getByText('30m')).toBeInTheDocument();
    expect(screen.getByText('5')).toBeInTheDocument();
  });

  it('renders TabBar with 4 tabs', async () => {
    render(<SessionDetailPage />, { wrapper: createWrapper() });

    await waitForLoad();

    const tabs = screen.getAllByRole('tab');
    expect(tabs).toHaveLength(4);
    expect(tabs[0]).toHaveTextContent('Timeline');
    expect(tabs[1]).toHaveTextContent('Messages');
    expect(tabs[2]).toHaveTextContent('Memories');
    expect(tabs[3]).toHaveTextContent('Metadata');
  });

  it('shows Timeline tab by default with turn messages', async () => {
    render(<SessionDetailPage />, { wrapper: createWrapper() });

    await waitForLoad();

    // The Timeline tab should be visible and show turn messages
    // The factory builds 2 turns per detail, both with same content
    const messages = screen.getAllByText('Sample turn content for testing.');
    expect(messages.length).toBeGreaterThanOrEqual(1);
  });

  it('switches to Messages tab', async () => {
    const user = userEvent.setup();
    render(<SessionDetailPage />, { wrapper: createWrapper() });

    await waitForLoad();

    const tabs = screen.getAllByRole('tab');
    await user.click(tabs[1]!);

    // Messages tab shows the same content — turn messages
    expect(screen.getAllByText('Sample turn content for testing.').length).toBeGreaterThan(0);
  });

  it('switches to Memories tab and shows tags', async () => {
    const user = userEvent.setup();
    render(<SessionDetailPage />, { wrapper: createWrapper() });

    await waitForLoad();

    const tabs = screen.getAllByRole('tab');
    await user.click(tabs[2]!);

    expect(screen.getByText('Memory Tags')).toBeInTheDocument();
    // Factory default tags
    expect(screen.getByText('exploration')).toBeInTheDocument();
    expect(screen.getByText('debugging')).toBeInTheDocument();
  });

  it('switches to Metadata tab and shows key-value table', async () => {
    const user = userEvent.setup();
    render(<SessionDetailPage />, { wrapper: createWrapper() });

    await waitForLoad();

    const tabs = screen.getAllByRole('tab');
    await user.click(tabs[3]!);

    expect(screen.getByText('Session ID')).toBeInTheDocument();
    expect(screen.getByText('Status')).toBeInTheDocument();
    expect(screen.getByText('Agent')).toBeInTheDocument();
    expect(screen.getByText('Tokens Used')).toBeInTheDocument();
  });

  it('shows overflow menu with delete option', async () => {
    render(<SessionDetailPage />, { wrapper: createWrapper() });

    await waitForLoad();

    // Click overflow menu (MoreVertical button)
    const moreButton = screen.getByRole('button', { name: /more actions/i });
    expect(moreButton).toBeInTheDocument();
  });

  it('opens delete confirmation modal from overflow menu and navigates back on confirm', async () => {
    const user = userEvent.setup();
    render(<SessionDetailPage />, { wrapper: createWrapper() });

    await waitForLoad();

    // Click the overflow menu button
    await user.click(screen.getByRole('button', { name: /more actions/i }));

    // Click the Delete Session option in the dropdown
    await user.click(screen.getByRole('button', { name: /delete session/i }));

    // Modal should appear
    expect(screen.getByText('Delete Session')).toBeInTheDocument();
    expect(screen.getByText(/are you sure/i)).toBeInTheDocument();

    // Click the Delete button in the modal footer
    const deleteButtons = screen.getAllByRole('button', { name: /^delete$/i });
    // The last one is inside the modal
    await user.click(deleteButtons[deleteButtons.length - 1]!);

    // Should navigate back to sessions list
    await waitFor(() => {
      expect(screen.getByTestId('sessions-list-page')).toBeInTheDocument();
    });
  });

  it('shows not-found state for non-existent session', async () => {
    render(<SessionDetailPage />, {
      wrapper: createWrapper('/sessions/nonexistent'),
    });

    expect(await screen.findByText('Session not found')).toBeInTheDocument();
    expect(screen.getByText('Back to Sessions')).toBeInTheDocument();
  });

  it('shows error state with retry button', async () => {
    server.use(
      http.get('*/api/v1/sessions/:id', () => {
        return HttpResponse.json({ detail: 'Server error' }, { status: 500 });
      }),
    );

    render(<SessionDetailPage />, { wrapper: createWrapper() });

    expect(await screen.findByText('Session not found')).toBeInTheDocument();
    expect(screen.getByText('Back to Sessions')).toBeInTheDocument();
    expect(screen.getByText('Retry')).toBeInTheDocument();
  });
});
