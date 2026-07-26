import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter, Route, Routes } from 'react-router';
import { http, HttpResponse } from 'msw';
import { describe, expect, it, beforeEach } from 'vitest';
import { SettingsPage } from './SettingsPage';
import { server } from '../../../tests/mocks/server';
import type { ReactNode } from 'react';

/* ─── Wrapper for react-query + router ─────────────────────── */

function createWrapper(initialRoute = '/settings/general') {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: 0,
      },
    },
  });

  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={[initialRoute]}>
          <Routes>
            <Route path="/settings" element={children} />
            <Route path="/settings/:section" element={children} />
          </Routes>
        </MemoryRouter>
      </QueryClientProvider>
    );
  };
}

/* ─── Tests ────────────────────────────────────────────────── */

describe('SettingsPage', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  it('renders the page header title', async () => {
    render(<SettingsPage />, { wrapper: createWrapper('/settings/general') });

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Settings' })).toBeInTheDocument();
    });
  });

  it('renders sidebar with all 8 section labels', async () => {
    render(<SettingsPage />, { wrapper: createWrapper('/settings/general') });

    await waitFor(() => {
      expect(screen.getByText('General')).toBeInTheDocument();
      expect(screen.getByText('Storage')).toBeInTheDocument();
      expect(screen.getByText('MCP Server')).toBeInTheDocument();
      expect(screen.getByText('LLM Providers')).toBeInTheDocument();
      expect(screen.getByText('Notifications')).toBeInTheDocument();
      expect(screen.getByText('Agents & Skills')).toBeInTheDocument();
      expect(screen.getByText('Analytics')).toBeInTheDocument();
      expect(screen.getByText('Data Management')).toBeInTheDocument();
    });
  });

  it('renders section content from API', async () => {
    render(<SettingsPage />, { wrapper: createWrapper('/settings/general') });

    await waitFor(() => {
      expect(screen.getByText('General Settings')).toBeInTheDocument();
    });

    // Settings fields from the MSW handler — labels are formatted (camelCase/snake → spaced)
    expect(screen.getByText(/^theme$/i)).toBeInTheDocument();
    expect(screen.getByText(/^language$/i)).toBeInTheDocument();

    // Boolean field: rendered as a checkbox with the label text "notifications enabled"
    expect(screen.getByText(/notifications enabled/i)).toBeInTheDocument();
  });

  it('defaults to general section when no section param is provided', async () => {
    render(<SettingsPage />, { wrapper: createWrapper('/settings') });

    await waitFor(() => {
      expect(screen.getByText('General Settings')).toBeInTheDocument();
    });
  });

  it('highlights the active section in the sidebar', async () => {
    render(<SettingsPage />, { wrapper: createWrapper('/settings/general') });

    await waitFor(() => {
      const activeLink = screen.getByRole('link', { name: /general/i });
      expect(activeLink).toHaveAttribute('aria-current', 'page');
    });

    // Non-active sections should not have aria-current
    const storageLink = screen.getByRole('link', { name: /storage/i });
    expect(storageLink).not.toHaveAttribute('aria-current');
  });

  it('navigates to a different section via sidebar link', async () => {
    render(<SettingsPage />, { wrapper: createWrapper('/settings/general') });

    await waitFor(() => {
      expect(screen.getByText('General Settings')).toBeInTheDocument();
    });

    // Click on the LLM Providers link
    const llmProvidersLink = screen.getByRole('link', { name: /LLM Providers/i });
    expect(llmProvidersLink).toHaveAttribute('href', '/settings/llm-providers');
  });

  it('shows error state for non-existent section', async () => {
    server.use(
      http.get('*/api/v1/settings/:section', () => {
        return HttpResponse.json({ detail: 'Section not found' }, { status: 404 });
      }),
    );

    render(<SettingsPage />, { wrapper: createWrapper('/settings/nonexistent') });

    await waitFor(() => {
      expect(screen.getByText(/failed to load settings/i)).toBeInTheDocument();
    });

    // Retry button should be present
    expect(screen.getByRole('button', { name: /retry/i })).toBeInTheDocument();
  });

  it('shows empty state when section has no configurable settings', async () => {
    server.use(
      http.get('*/api/v1/settings/:section', () => {
        return HttpResponse.json({
          key: 'appearance',
          label: 'Appearance',
          settings: {},
        });
      }),
    );

    render(<SettingsPage />, { wrapper: createWrapper('/settings/appearance') });

    await waitFor(() => {
      expect(screen.getByText(/no settings/i)).toBeInTheDocument();
    });
  });

  it('shows editable text input for string values', async () => {
    render(<SettingsPage />, { wrapper: createWrapper('/settings/general') });

    await waitFor(() => {
      // General settings has 'theme' (string) and 'language' (string)
      expect(screen.getByDisplayValue('dark')).toBeInTheDocument();
      expect(screen.getByDisplayValue('en')).toBeInTheDocument();
    });
  });

  it('shows checkbox toggle for boolean values', async () => {
    render(<SettingsPage />, { wrapper: createWrapper('/settings/general') });

    await waitFor(() => {
      // notifications_enabled is boolean
      const checkbox = screen.getByRole('checkbox', { name: /notifications_enabled/i });
      expect(checkbox).toBeInTheDocument();
      expect(checkbox).toBeChecked();
    });
  });

  it('shows save and discard buttons when a value is changed', async () => {
    const user = userEvent.setup();

    render(<SettingsPage />, { wrapper: createWrapper('/settings/general') });

    await waitFor(() => {
      expect(screen.getByDisplayValue('dark')).toBeInTheDocument();
    });

    // Change a value
    const themeInput = screen.getByDisplayValue('dark');
    await user.clear(themeInput);
    await user.type(themeInput, 'light');

    // Save and Discard buttons should now be visible
    expect(screen.getByRole('button', { name: /save changes/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /discard/i })).toBeInTheDocument();
  });

  it('discards changes when discard button is clicked', async () => {
    const user = userEvent.setup();

    render(<SettingsPage />, { wrapper: createWrapper('/settings/general') });

    await waitFor(() => {
      expect(screen.getByDisplayValue('dark')).toBeInTheDocument();
    });

    const themeInput = screen.getByDisplayValue('dark');
    await user.clear(themeInput);
    await user.type(themeInput, 'light');

    // Click discard
    const discardButton = screen.getByRole('button', { name: /discard/i });
    await user.click(discardButton);

    // Value should revert to original
    expect(screen.getByDisplayValue('dark')).toBeInTheDocument();

    // Save/discard buttons should no longer be visible
    expect(screen.queryByRole('button', { name: /save changes/i })).not.toBeInTheDocument();
  });
});
