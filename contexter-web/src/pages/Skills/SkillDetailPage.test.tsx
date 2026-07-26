import { act, render, screen, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter, Route, Routes } from 'react-router';
import { http, HttpResponse } from 'msw';
import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import { SkillDetailPage } from './SkillDetailPage';
import { server } from '../../../tests/mocks/server';
import type { ReactNode } from 'react';

// Polyfill ResizeObserver for Recharts in jsdom
class ResizeObserverStub {
  observe() {}
  unobserve() {}
  disconnect() {}
}

beforeAll(() => {
  window.ResizeObserver = ResizeObserverStub as unknown as typeof ResizeObserver;
});

afterAll(() => {
  delete (window as { ResizeObserver?: unknown }).ResizeObserver;
});

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  });

  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={['/skills/skl_000001']}>
          <Routes>
            <Route path="/skills/:id" element={children} />
            <Route path="/skills" element={<div data-testid="registry-page" />} />
          </Routes>
        </MemoryRouter>
      </QueryClientProvider>
    );
  };
}

describe('SkillDetailPage', () => {
  it('renders loading state while fetching', async () => {
    render(<SkillDetailPage />, { wrapper: createWrapper() });

    const skeletons = screen.getAllByTestId('skeleton');
    expect(skeletons.length).toBeGreaterThan(0);
  });

  it('renders skill name and category from API', async () => {
    render(<SkillDetailPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Review Pro' })).toBeInTheDocument();
    });

    // Category badge appears in info cards section (may also appear in overview tab)
    const badges = screen.getAllByText('code-review');
    expect(badges.length).toBeGreaterThan(0);
  });

  it('renders breadcrumb with Skills > skill name', async () => {
    render(<SkillDetailPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Review Pro' })).toBeInTheDocument();
    });

    // Breadcrumb should show "Skills" link
    expect(screen.getByText('Skills')).toBeInTheDocument();
  });

  it('renders effectiveness score and usage count stat cards', async () => {
    render(<SkillDetailPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Effectiveness Score')).toBeInTheDocument();
    });

    expect(screen.getByText('Usage Count')).toBeInTheDocument();
  });

  it('renders three tabs: Overview, Usage, Versions', async () => {
    render(<SkillDetailPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Overview')).toBeInTheDocument();
    });

    expect(screen.getByText('Usage')).toBeInTheDocument();
    expect(screen.getByText('Versions')).toBeInTheDocument();
  });

  it('shows Overview tab content by default', async () => {
    render(<SkillDetailPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Overview')).toBeInTheDocument();
    });

    // Overview tab panel should show "Category" heading (appears in both info cards and overview panel)
    const categoryElements = screen.getAllByText('Category');
    expect(categoryElements.length).toBeGreaterThanOrEqual(1);

    // Created date should be visible
    expect(screen.getByText(/july/i)).toBeInTheDocument();
  });

  it('switches to Usage tab and shows trend chart', async () => {
    render(<SkillDetailPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Usage')).toBeInTheDocument();
    });

    // Click the Usage tab
    const usageTab = screen.getByRole('tab', { name: /usage/i });
    await act(async () => {
      usageTab.click();
    });

    // The chart container should render (Recharts needs real layout to render SVG in jsdom)
    await waitFor(() => {
      const chartContainer = document.querySelector('.recharts-responsive-container');
      expect(chartContainer).not.toBeNull();
    });

    // The trend heading should appear
    expect(screen.getByText('Usage Trend')).toBeInTheDocument();
  });

  it('switches to Versions tab and shows session data', async () => {
    render(<SkillDetailPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Versions')).toBeInTheDocument();
    });

    // Click the Versions tab
    const versionsTab = screen.getByRole('tab', { name: 'Versions' });
    await act(async () => {
      versionsTab.click();
    });

    // Should show session table
    await waitFor(() => {
      const table = document.querySelector('table');
      expect(table).not.toBeNull();
    });
  });

  it('shows error state when skill is not found', async () => {
    server.use(
      http.get('*/api/v1/skills/:id', () => {
        return HttpResponse.json({ detail: 'Skill not found' }, { status: 404 });
      }),
    );

    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });

    render(
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={['/skills/skl_nonexistent']}>
          <Routes>
            <Route path="/skills/:id" element={<SkillDetailPage />} />
          </Routes>
        </MemoryRouter>
      </QueryClientProvider>,
    );

    await waitFor(() => {
      expect(screen.getByText(/not found/i)).toBeInTheDocument();
    });
  });
});
