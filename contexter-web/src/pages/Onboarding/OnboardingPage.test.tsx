import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, beforeEach } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router';
import { http, HttpResponse } from 'msw';
import type { ReactNode } from 'react';
import { OnboardingPage } from './OnboardingPage';
import { server } from '../../../tests/mocks/server';

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={['/onboarding']}>{children}</MemoryRouter>
      </QueryClientProvider>
    );
  };
}

describe('OnboardingPage', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  it('renders the page header title', async () => {
    render(<OnboardingPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /onboarding/i })).toBeInTheDocument();
    });
  });

  it('renders onboarding steps', async () => {
    render(<OnboardingPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByText('Welcome')).toBeInTheDocument();
      expect(screen.getByText('Connect Provider')).toBeInTheDocument();
      expect(screen.getByText('First Session')).toBeInTheDocument();
    });
  });

  it('shows loading state while data is loading', () => {
    render(<OnboardingPage />, { wrapper: createWrapper() });
    expect(screen.getAllByTestId('skeleton').length).toBeGreaterThan(0);
  });

  it('shows completion message when onboarding is complete', async () => {
    server.use(
      http.get('*/api/v1/onboarding/status', () => HttpResponse.json({
        current_step: 4,
        total_steps: 4,
        completed: true,
        steps: [
          { id: 'welcome', label: 'Welcome', completed: true },
          { id: 'connect', label: 'Connect Provider', completed: true },
          { id: 'first-session', label: 'First Session', completed: true },
          { id: 'explore', label: 'Explore Dashboard', completed: true },
        ],
      })),
    );
    render(<OnboardingPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /onboarding complete/i })).toBeInTheDocument();
    });
  });
});
