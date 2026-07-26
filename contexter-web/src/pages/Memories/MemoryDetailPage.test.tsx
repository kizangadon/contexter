import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter, Route, Routes } from 'react-router';
import { describe, expect, it } from 'vitest';
import { MemoryDetailPage } from './MemoryDetailPage';

function createWrapper(id: string) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });

  return function Wrapper() {
    return (
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={[`/memories/${id}`]}>
          <Routes>
            <Route path="/memories/:id" element={<MemoryDetailPage />} />
            <Route path="/memories" element={<div>Memories list</div>} />
          </Routes>
        </MemoryRouter>
      </QueryClientProvider>
    );
  };
}

describe('MemoryDetailPage', () => {
  it('renders page header with breadcrumb', async () => {
    render(<MemoryDetailPage />, { wrapper: createWrapper('mem_000001') });

    await waitFor(() => {
      expect(screen.getByText('Memory')).toBeInTheDocument();
    });
  });

  it('renders the memory content', async () => {
    render(<MemoryDetailPage />, { wrapper: createWrapper('mem_000001') });

    await waitFor(() => {
      const elements = screen.getAllByText(/A key insight about the system architecture/);
      expect(elements.length).toBe(2);
    });
  });

  it('renders memory type badge', async () => {
    render(<MemoryDetailPage />, { wrapper: createWrapper('mem_000001') });

    await waitFor(() => {
      expect(screen.getAllByText('conversation').length).toBeGreaterThanOrEqual(1);
    });
  });

  it('renders confidence bar', async () => {
    render(<MemoryDetailPage />, { wrapper: createWrapper('mem_000001') });

    await waitFor(() => {
      expect(screen.getByText(/85%/)).toBeInTheDocument();
    });
  });

  it('renders tags', async () => {
    render(<MemoryDetailPage />, { wrapper: createWrapper('mem_000001') });

    await waitFor(() => {
      expect(screen.getByText('architecture')).toBeInTheDocument();
      expect(screen.getByText('insight')).toBeInTheDocument();
    });
  });

  it('renders Content tab by default', async () => {
    render(<MemoryDetailPage />, { wrapper: createWrapper('mem_000001') });

    await waitFor(() => {
      const tab = screen.getByRole('tab', { name: 'Content' });
      expect(tab).toHaveAttribute('aria-selected', 'true');
    });
  });

  it('switches to Versions tab when clicked', async () => {
    const user = userEvent.setup();
    render(<MemoryDetailPage />, { wrapper: createWrapper('mem_000001') });

    await waitFor(() => {
      expect(screen.getByRole('tab', { name: 'Content' })).toBeInTheDocument();
    });

    const versionsTab = screen.getByRole('tab', { name: 'Versions' });
    await user.click(versionsTab);

    expect(versionsTab).toHaveAttribute('aria-selected', 'true');
  });

  it('switches to Related tab when clicked', async () => {
    const user = userEvent.setup();
    render(<MemoryDetailPage />, { wrapper: createWrapper('mem_000001') });

    await waitFor(() => {
      expect(screen.getByRole('tab', { name: 'Content' })).toBeInTheDocument();
    });

    const relatedTab = screen.getByRole('tab', { name: 'Related' });
    await user.click(relatedTab);

    expect(relatedTab).toHaveAttribute('aria-selected', 'true');
  });

  it('renders source session link', async () => {
    render(<MemoryDetailPage />, { wrapper: createWrapper('mem_000001') });

    await waitFor(() => {
      const links = screen.getAllByRole('link');
      const sessionLink = links.find((l) => l.getAttribute('href')?.includes('/sessions/'));
      expect(sessionLink).toBeDefined();
    });
  });

  it('shows loading skeleton while fetching', () => {
    const { container } = render(<MemoryDetailPage />, {
      wrapper: createWrapper('mem_000001'),
    });

    const skeletons = container.querySelectorAll('[data-testid="skeleton"]');
    expect(skeletons.length).toBeGreaterThan(0);
  });

  it('shows not found state for non-existent memory', async () => {
    render(<MemoryDetailPage />, { wrapper: createWrapper('mem_nonexistent') });

    await waitFor(() => {
      expect(screen.getByText('Memory not found')).toBeInTheDocument();
    });
  });

  it('shows go back link in not found state', async () => {
    render(<MemoryDetailPage />, { wrapper: createWrapper('mem_nonexistent') });

    await waitFor(() => {
      expect(screen.getByText('Back to memories')).toBeInTheDocument();
    });
  });
});
