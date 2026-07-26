import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, beforeEach } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router';
import type { ReactNode } from 'react';
import { FeedbackPage } from './FeedbackPage';
import { server } from '../../../tests/mocks/server';

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={['/feedback']}>{children}</MemoryRouter>
      </QueryClientProvider>
    );
  };
}

describe('FeedbackPage', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  it('renders the page header title', async () => {
    render(<FeedbackPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /feedback/i })).toBeInTheDocument();
    });
  });

  it('renders all three tabs', async () => {
    render(<FeedbackPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByRole('tab', { name: /changelog/i })).toBeInTheDocument();
      expect(screen.getByRole('tab', { name: /report bug/i })).toBeInTheDocument();
      expect(screen.getByRole('tab', { name: /suggest/i })).toBeInTheDocument();
    });
  });

  it('shows changelog entries by default', async () => {
    render(<FeedbackPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText(/v1\.1\.0/)).toBeInTheDocument();
    });
  });

  it('shows bug form when Report Bug tab is clicked', async () => {
    render(<FeedbackPage />, { wrapper: createWrapper() });
    const bugTab = await screen.findByRole('tab', { name: /report bug/i });
    await userEvent.click(bugTab);
    await waitFor(() => {
      expect(screen.getByLabelText(/title/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/description/i)).toBeInTheDocument();
    });
  });

  it('shows suggestion form when Suggest tab is clicked', async () => {
    render(<FeedbackPage />, { wrapper: createWrapper() });
    const suggestTab = await screen.findByRole('tab', { name: /suggest/i });
    await userEvent.click(suggestTab);
    await waitFor(() => {
      expect(screen.getByLabelText(/title/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/description/i)).toBeInTheDocument();
    });
  });

  it('shows loading skeletons while data is loading', () => {
    render(<FeedbackPage />, { wrapper: createWrapper() });
    expect(screen.getAllByTestId('skeleton').length).toBeGreaterThan(0);
  });
});
