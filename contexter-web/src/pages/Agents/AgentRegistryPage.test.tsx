import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import { MemoryRouter } from 'react-router';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import type { ReactNode } from 'react';
import { AgentRegistryPage } from './AgentRegistryPage';

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  });

  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <MemoryRouter>
        <QueryClientProvider client={queryClient}>
          {children}
        </QueryClientProvider>
      </MemoryRouter>
    );
  };
}

describe('AgentRegistryPage', () => {
  it('renders the page title', async () => {
    render(<AgentRegistryPage />, { wrapper: createWrapper() });

    expect(screen.getByText('Agents')).toBeInTheDocument();
  });

  it('renders FilterBar with status options', async () => {
    render(<AgentRegistryPage />, { wrapper: createWrapper() });

    // The FilterBar renders a "Status" label and select
    expect(screen.getByText('Status')).toBeInTheDocument();
    expect(screen.getByLabelText('Status')).toBeInTheDocument();
  });

  it('renders agent cards after loading', async () => {
    render(<AgentRegistryPage />, { wrapper: createWrapper() });

    // Wait for data to load — the MSW handler returns 2 agents
    await screen.findAllByRole('button', {
      name: /agent/i,
    });
    // AgentCard renders as role="button" — but there's also the FilterBar select
    // We look for cards that contain agent names
    expect(screen.getByText('Agent-1')).toBeInTheDocument();
    expect(screen.getByText('Agent-2')).toBeInTheDocument();
  });

  it('shows loading skeleton initially', () => {
    render(<AgentRegistryPage />, { wrapper: createWrapper() });

    // LoadingSkeleton renders elements with data-testid="skeleton"
    const skeletons = screen.getAllByTestId('skeleton');
    expect(skeletons.length).toBeGreaterThan(0);
  });

  it('filters agents by status when filter changes', async () => {
    const user = userEvent.setup();
    render(<AgentRegistryPage />, { wrapper: createWrapper() });

    // Wait for data to load
    await screen.findByText('Agent-1');

    // Change status filter to "active" — MSW handler filters by status
    const statusSelect = screen.getByLabelText('Status');
    await user.selectOptions(statusSelect, 'active');

    // Should still show active agents
    // MSW returns all agents filtered by status
    // The agents are seeded as 'active' by default
    await screen.findByText('Agent-1');
  });

  it('shows empty state when no agents match filter', async () => {
    const user = userEvent.setup();
    render(<AgentRegistryPage />, { wrapper: createWrapper() });

    await screen.findByText('Agent-1');

    // Change to a status that no agents have
    const statusSelect = screen.getByLabelText('Status');
    await user.selectOptions(statusSelect, 'offline');

    // Should show empty state
    expect(await screen.findByText('No agents found')).toBeInTheDocument();
  });
});
