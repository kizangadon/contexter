import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter, Route, Routes } from 'react-router';
import { http, HttpResponse } from 'msw';
import { describe, expect, it } from 'vitest';
import { SkillRegistryPage } from './SkillRegistryPage';
import { server } from '../../../tests/mocks/server';
import type { ReactNode } from 'react';

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  });

  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={['/skills']}>
          <Routes>
            <Route path="/skills" element={children} />
            <Route path="/skills/:id" element={<div data-testid="detail-page" />} />
          </Routes>
        </MemoryRouter>
      </QueryClientProvider>
    );
  };
}

describe('SkillRegistryPage', () => {
  it('renders loading skeleton while fetching', async () => {
    render(<SkillRegistryPage />, { wrapper: createWrapper() });

    // Should show skeleton cards initially
    const skeletons = screen.getAllByTestId('skeleton');
    expect(skeletons.length).toBeGreaterThan(0);
  });

  it('renders skill cards from API', async () => {
    render(<SkillRegistryPage />, { wrapper: createWrapper() });

    // Wait for data to load
    await waitFor(() => {
      expect(screen.getByText('Review Pro')).toBeInTheDocument();
    });

    expect(screen.getByText('Bug Hunter')).toBeInTheDocument();
    expect(screen.getByText('Refactor Master')).toBeInTheDocument();
  });

  it('renders page title "Skills"', async () => {
    render(<SkillRegistryPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Skills')).toBeInTheDocument();
    });
  });

  it('filters by category when filter changes', async () => {
    render(<SkillRegistryPage />, { wrapper: createWrapper() });

    // Wait for data to load
    await waitFor(() => {
      expect(screen.getByText('Review Pro')).toBeInTheDocument();
    });

    // Get the category filter select and pick 'code-review'
    const filterSelect = screen.getByLabelText('Category');
    await userEvent.selectOptions(filterSelect, 'code-review');

    // Only 'Review Pro' (code-review category) should show
    await waitFor(() => {
      expect(screen.getByText('Review Pro')).toBeInTheDocument();
    });

    // Bug Hunter (debugging) and Refactor Master (refactoring) should not appear
    expect(screen.queryByText('Bug Hunter')).not.toBeInTheDocument();
    expect(screen.queryByText('Refactor Master')).not.toBeInTheDocument();
  });

  it('shows empty state when no skills match filter', async () => {
    // Override the skills handler to return empty data
    server.use(
      http.get('*/api/v1/skills', () => {
        return HttpResponse.json([]);
      }),
    );

    render(<SkillRegistryPage />, { wrapper: createWrapper() });

    // Should show empty state
    await waitFor(() => {
      expect(screen.getByText('No skills found')).toBeInTheDocument();
    });
  });

  it('navigates to skill detail on card click', async () => {
    render(<SkillRegistryPage />, { wrapper: createWrapper() });

    // Wait for data
    await waitFor(() => {
      expect(screen.getByText('Review Pro')).toBeInTheDocument();
    });

    // Click the first skill card
    const cards = screen.getAllByTestId('skill-card');
    await userEvent.click(cards[0]!);

    // Should navigate to detail page
    await waitFor(() => {
      expect(screen.getByTestId('detail-page')).toBeInTheDocument();
    });
  });
});
