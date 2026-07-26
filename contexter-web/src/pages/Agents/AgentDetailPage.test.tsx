import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, beforeAll } from 'vitest';
import { MemoryRouter, Route, Routes } from 'react-router';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { AgentDetailPage } from './AgentDetailPage';

// ResizeObserver polyfill for Recharts ResponsiveContainer in jsdom
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

function createWrapper(initialRoute = '/agents/agt_000001') {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  });

  return function Wrapper() {
    return (
      <MemoryRouter initialEntries={[initialRoute]}>
        <QueryClientProvider client={queryClient}>
          <Routes>
            <Route path="/agents/:id" element={<AgentDetailPage />} />
          </Routes>
        </QueryClientProvider>
      </MemoryRouter>
    );
  };
}

/* Helper: wait until data is loaded and the h1 is rendered */
async function waitForLoad() {
  await screen.findByRole('heading', { level: 1 });
}

describe('AgentDetailPage', () => {
  it('renders loading skeleton while fetching', () => {
    render(<AgentDetailPage />, { wrapper: createWrapper() });

    const skeletons = screen.getAllByTestId('skeleton');
    expect(skeletons.length).toBeGreaterThan(0);
  });

  it('renders agent name in h1 after loading', async () => {
    render(<AgentDetailPage />, { wrapper: createWrapper() });

    const heading = await screen.findByRole('heading', { level: 1 });
    expect(heading).toHaveTextContent('Agent-1');
  });

  it('renders breadcrumb trail with Agents link', async () => {
    render(<AgentDetailPage />, { wrapper: createWrapper() });

    await waitForLoad();

    const nav = screen.getByLabelText('Breadcrumb');
    expect(within(nav).getByText('Agents')).toBeInTheDocument();
    expect(within(nav).getByText('Agent-1')).toBeInTheDocument();
  });

  it('renders status badge with agent status', async () => {
    render(<AgentDetailPage />, { wrapper: createWrapper() });

    await waitForLoad();
    expect(screen.getByText('active')).toBeInTheDocument();
  });

  it('renders capability tags in info header', async () => {
    render(<AgentDetailPage />, { wrapper: createWrapper() });

    await waitForLoad();

    // Tags appear in the info header section — use getAllByText since they
    // appear both in header and overview tab
    const codeReviewTags = screen.getAllByText('code-review');
    expect(codeReviewTags.length).toBeGreaterThanOrEqual(1);
  });

  it('renders efficiency score highlighted', async () => {
    render(<AgentDetailPage />, { wrapper: createWrapper() });

    await waitForLoad();
    expect(screen.getByText('Efficiency Score')).toBeInTheDocument();
    expect(screen.getByText('85%')).toBeInTheDocument();
  });

  it('renders TabBar with all four tabs', async () => {
    render(<AgentDetailPage />, { wrapper: createWrapper() });

    await waitForLoad();

    // Use getAllByRole to find tabs
    const tabs = screen.getAllByRole('tab');
    expect(tabs).toHaveLength(4);
    expect(tabs[0]).toHaveTextContent('Overview');
    expect(tabs[1]).toHaveTextContent('Sessions');
    expect(tabs[2]).toHaveTextContent('Skills');
    expect(tabs[3]).toHaveTextContent('Version History');
  });

  it('shows Overview tab by default with stats', async () => {
    render(<AgentDetailPage />, { wrapper: createWrapper() });

    await waitForLoad();

    // StatCard values
    expect(screen.getByText('42')).toBeInTheDocument();
    expect(screen.getByText('320ms')).toBeInTheDocument();
  });

  it('switches to Sessions tab and shows session table', async () => {
    const user = userEvent.setup();
    render(<AgentDetailPage />, { wrapper: createWrapper() });

    await waitForLoad();

    // Click the Sessions tab button
    const tabs = screen.getAllByRole('tab');
    await user.click(tabs[1]!);

    expect(screen.getByText('Recent Sessions')).toBeInTheDocument();
  });

  it('switches to Skills tab and shows chart', async () => {
    const user = userEvent.setup();
    render(<AgentDetailPage />, { wrapper: createWrapper() });

    await waitForLoad();

    const tabs = screen.getAllByRole('tab');
    await user.click(tabs[2]!);

    expect(screen.getByText('Efficiency Trend')).toBeInTheDocument();
  });

  it('switches to Version History tab and shows config table', async () => {
    const user = userEvent.setup();
    render(<AgentDetailPage />, { wrapper: createWrapper() });

    await waitForLoad();

    const tabs = screen.getAllByRole('tab');
    await user.click(tabs[3]!);

    expect(screen.getByText('Configuration')).toBeInTheDocument();
  });

  it('shows 404 error for non-existent agent', async () => {
    render(<AgentDetailPage />, {
      wrapper: createWrapper('/agents/agt_nonexistent'),
    });

    expect(await screen.findByText('Agent not found')).toBeInTheDocument();
  });

  it('shows back button on 404 error', async () => {
    render(<AgentDetailPage />, {
      wrapper: createWrapper('/agents/agt_nonexistent'),
    });

    expect(
      await screen.findByText('Back to Agents'),
    ).toBeInTheDocument();
  });
});
