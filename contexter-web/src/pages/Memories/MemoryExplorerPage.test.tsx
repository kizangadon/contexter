import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router';
import { describe, expect, it } from 'vitest';
import { MemoryExplorerPage } from './MemoryExplorerPage';

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });

  return function Wrapper({ children }: { children: React.ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <MemoryRouter>{children}</MemoryRouter>
      </QueryClientProvider>
    );
  };
}

describe('MemoryExplorerPage', () => {
  it('renders the page title', async () => {
    render(<MemoryExplorerPage />, { wrapper: createWrapper() });

    expect(screen.getByText('Memory Explorer')).toBeInTheDocument();
  });

  it('shows loading skeleton while fetching memories', () => {
    const { container } = render(<MemoryExplorerPage />, { wrapper: createWrapper() });

    const skeletons = container.querySelectorAll('[data-testid="skeleton"]');
    expect(skeletons.length).toBeGreaterThan(0);
  });

  it('renders memories data in the table', async () => {
    render(<MemoryExplorerPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      const cells = screen.getAllByText(/A key insight about the system architecture/);
      expect(cells.length).toBeGreaterThanOrEqual(1);
    });
  });

  it('renders type badges for each memory', async () => {
    render(<MemoryExplorerPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      const badges = screen.getAllByText('conversation');
      expect(badges.length).toBeGreaterThan(0);
    });
  });

  it('filters by memory type when filter changes', async () => {
    const user = userEvent.setup();
    render(<MemoryExplorerPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      const cells = screen.getAllByText(/A key insight/);
      expect(cells.length).toBeGreaterThanOrEqual(1);
    });

    const select = screen.getByLabelText('Memory Type');
    await user.selectOptions(select, 'decision');

    await waitFor(() => {
      const insights = screen.queryAllByText(/system architecture/);
      expect(insights.length).toBe(0);
    });
  });

  it('shows empty state when no memories match', async () => {
    const user = userEvent.setup();
    render(<MemoryExplorerPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      const cells = screen.getAllByText(/A key insight/);
      expect(cells.length).toBeGreaterThanOrEqual(1);
    });

    const select = screen.getByLabelText('Memory Type');
    await user.selectOptions(select, 'custom');

    await waitFor(() => {
      expect(screen.getByText('No memories found')).toBeInTheDocument();
    });
  });

  it('navigates to memory detail on row click', async () => {
    render(<MemoryExplorerPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      const cells = screen.getAllByText(/A key insight/);
      expect(cells.length).toBeGreaterThanOrEqual(1);
    });

    const rows = screen.getAllByRole('row');
    expect(rows.length).toBeGreaterThan(1);
  });

  it('renders tag badges for memories', async () => {
    render(<MemoryExplorerPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      const tags = screen.getAllByText('architecture');
      expect(tags.length).toBeGreaterThanOrEqual(1);
    });
  });

  it('shows confidence values', async () => {
    render(<MemoryExplorerPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      const values = screen.getAllByText('85%');
      expect(values.length).toBeGreaterThanOrEqual(1);
    });
  });

  it('shows version numbers', async () => {
    render(<MemoryExplorerPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      const versions = screen.getAllByText(/^v1$/);
      expect(versions.length).toBeGreaterThanOrEqual(1);
    });
  });
});
