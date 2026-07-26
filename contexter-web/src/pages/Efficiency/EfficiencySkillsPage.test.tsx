import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, beforeEach } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router';
import { http, HttpResponse } from 'msw';
import type { ReactNode } from 'react';
import { EfficiencySkillsPage } from './EfficiencySkillsPage';
import { server } from '../../../tests/mocks/server';

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={['/efficiency/skills']}>
          {children}
        </MemoryRouter>
      </QueryClientProvider>
    );
  };
}

describe('EfficiencySkillsPage', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  it('renders the page header title', async () => {
    render(<EfficiencySkillsPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Skill Effectiveness' })).toBeInTheDocument();
    });
  });

  it('renders stat cards with skill metrics', async () => {
    render(<EfficiencySkillsPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Total Skills')).toBeInTheDocument();
      expect(screen.getByText('Avg Score')).toBeInTheDocument();
      expect(screen.getByText('Total Usage')).toBeInTheDocument();
    });
  });

  it('renders skill data in table', async () => {
    render(<EfficiencySkillsPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Review Pro')).toBeInTheDocument();
      expect(screen.getByText('Bug Hunter')).toBeInTheDocument();
      expect(screen.getByText('Refactor Master')).toBeInTheDocument();
    });
  });

  it('shows loading skeletons while loading', () => {
    render(<EfficiencySkillsPage />, { wrapper: createWrapper() });
    expect(screen.getAllByTestId('skeleton').length).toBeGreaterThan(0);
  });

  it('shows error state when API fails', async () => {
    server.use(
      http.get('*/api/v1/efficiency/skills', () => {
        return HttpResponse.json({ detail: 'Error' }, { status: 500 });
      }),
    );
    render(<EfficiencySkillsPage />, { wrapper: createWrapper() });
    expect(await screen.findByRole('button', { name: /retry/i })).toBeInTheDocument();
    expect(screen.getByText('Failed to load skill data')).toBeInTheDocument();
  });
});
