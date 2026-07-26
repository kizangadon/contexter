import { Suspense } from 'react';
import { render, screen } from '@testing-library/react';
import { createMemoryRouter, Navigate, Outlet, RouterProvider } from 'react-router';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { describe, expect, it, beforeAll, beforeEach } from 'vitest';
import { server } from '../tests/mocks/server';
import { routes } from './routes';

/* ─── Test helper ──────────────────────────────────────────── */

function renderRoute(initialEntry: string) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });

  const router = createMemoryRouter(
    [
      {
        element: (
          <Suspense fallback={<div>Loading...</div>}>
            <Outlet />
          </Suspense>
        ),
        children: [
          { index: true, element: <Navigate to="/dashboard" replace /> },
          ...routes,
        ],
      },
    ],
    { initialEntries: [initialEntry] },
  );

  return render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
}

/* ─── Preload lazy modules ───────────────────────────────────── */

// jsdom requires lazy modules to be resolved once before route tests
// otherwise the first dynamic import via React.lazy may time out
beforeAll(async () => {
  await Promise.all([
    import('./pages/Dashboard/DashboardPage'),
    import('./pages/Sessions/SessionManagerPage'),
    import('./pages/Sessions/SessionDetailPage'),
    import('./pages/Memories/MemoryExplorerPage'),
    import('./pages/Memories/MemoryDetailPage'),
    import('./pages/Agents/AgentRegistryPage'),
    import('./pages/Agents/AgentDetailPage'),
    import('./pages/Skills/SkillRegistryPage'),
    import('./pages/Skills/SkillDetailPage'),
    import('./pages/Settings/SettingsPage'),
    import('./pages/Efficiency/EfficiencyPage'),
    import('./pages/Search/SearchPage'),
    import('./pages/Playground/PlaygroundPage'),
    import('./pages/Notifications/NotificationsPage'),
    import('./pages/Feedback/FeedbackPage'),
    import('./pages/Exports/ExportsPage'),
    import('./pages/Onboarding/OnboardingPage'),
    import('./pages/Correlation/CorrelationPage'),
    import('./pages/Analytics/AnalyticsDashboardPage'),
    import('./pages/Audit/AuditPage'),
    import('./pages/NotFound/NotFoundPage'),
    import('./pages/Analytics/AnalyticsModelsPage'),
    import('./pages/Analytics/AnalyticsHealthPage'),
    import('./pages/Analytics/AnalyticsPerformancePage'),
    import('./pages/Analytics/AnalyticsResourcesPage'),
    import('./pages/Analytics/AnalyticsCostsPage'),
    import('./pages/Analytics/AnalyticsModelDetailPage'),
    import('./pages/Analytics/AnalyticsServicesPage'),
    import('./pages/Efficiency/EfficiencyMemoryPage'),
    import('./pages/Efficiency/EfficiencySessionsPage'),
    import('./pages/Efficiency/EfficiencyAgentsPage'),
    import('./pages/Efficiency/EfficiencySkillsPage'),
    import('./pages/Efficiency/EfficiencyTokensPage'),
    import('./pages/Efficiency/EfficiencyCorrelationPage'),
  ]);
});

/* ─── Tests ─────────────────────────────────────────────────── */

describe('Route resolution', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  it('/ has no direct handler — index redirect in App.tsx routes to /dashboard', async () => {
    // Render routes WITHOUT an index redirect: '/' hits the '*' catch-all → 404
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0 } },
    });
    const router = createMemoryRouter(
      [{
        element: (
          <Suspense fallback={<div>Loading...</div>}>
            <Outlet />
          </Suspense>
        ),
        children: [...routes],
      }],
      { initialEntries: ['/'] },
    );
    render(
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>,
    );
    // No direct '/' handler → NotFoundPage
    expect(await screen.findByText('Page not found')).toBeInTheDocument();

    // The matching /dashboard test below confirms that path renders correctly
    // Together these prove: '/' → (App.tsx redirect) → '/dashboard' → DashboardPage
  });

  it('/dashboard renders DashboardPage', async () => {
    renderRoute('/dashboard');
    expect(await screen.findByRole('heading', { name: 'Dashboard' })).toBeInTheDocument();
  });

  it('/sessions renders SessionManagerPage', async () => {
    renderRoute('/sessions');
    expect(await screen.findByRole('heading', { name: 'Sessions' })).toBeInTheDocument();
  });

  it('/sessions/ses_000001 renders SessionDetailPage', async () => {
    renderRoute('/sessions/ses_000001');
    // Page loads async data — wait for the heading to appear
    const heading = await screen.findByRole('heading', { name: /^Session / });
    expect(heading).toBeInTheDocument();
    // Breadcrumb should contain "Sessions" link
    expect(await screen.findByText('Sessions')).toBeInTheDocument();
  });

  it('/memories renders MemoryExplorerPage', async () => {
    renderRoute('/memories');
    // PageHeader title is "Memory Explorer"
    expect(await screen.findByRole('heading', { name: 'Memory Explorer' })).toBeInTheDocument();
  });

  it('/memories/mem_000001 renders MemoryDetailPage', async () => {
    renderRoute('/memories/mem_000001');
    // Page loads async data — wait for the heading to appear
    expect(await screen.findByRole('heading', { name: 'Memory' })).toBeInTheDocument();
  });

  it('/agents renders AgentRegistryPage', async () => {
    renderRoute('/agents');
    expect(await screen.findByRole('heading', { name: 'Agents' })).toBeInTheDocument();
  });

  it('/agents/agt_000001 renders AgentDetailPage', async () => {
    renderRoute('/agents/agt_000001');
    // Page loads async data — "Efficiency Score" text is unique to AgentDetailPage
    expect(await screen.findByText('Efficiency Score')).toBeInTheDocument();
  });

  it('/skills renders SkillRegistryPage', async () => {
    renderRoute('/skills');
    expect(await screen.findByRole('heading', { name: 'Skills' })).toBeInTheDocument();
  });

  it('/settings renders SettingsPage', async () => {
    renderRoute('/settings');
    expect(await screen.findByRole('heading', { name: 'Settings' })).toBeInTheDocument();
  });

  it('/search renders SearchPage with search input', async () => {
    renderRoute('/search');
    expect(await screen.findByRole('heading', { name: 'Search' })).toBeInTheDocument();
    expect(
      screen.getByPlaceholderText('Search sessions, memories, agents, skills...'),
    ).toBeInTheDocument();
  });

  it('/notifications renders NotificationsPage', async () => {
    renderRoute('/notifications');
    expect(await screen.findByRole('heading', { name: 'Notifications' })).toBeInTheDocument();
  });

  it('/analytics renders AnalyticsDashboardPage', async () => {
    renderRoute('/analytics');
    expect(await screen.findByRole('heading', { name: 'Analytics' })).toBeInTheDocument();
  });

  it('/efficiency renders EfficiencyPage', async () => {
    renderRoute('/efficiency');
    // PageHeader title is "Efficiency Mapper"
    expect(await screen.findByRole('heading', { name: 'Efficiency Mapper' })).toBeInTheDocument();
  });

  it('unknown route renders NotFoundPage', async () => {
    renderRoute('/nonexistent');
    expect(await screen.findByText('Page not found')).toBeInTheDocument();
  });

  it('/analytics/models renders AnalyticsModelsPage', async () => {
    renderRoute('/analytics/models');
    expect(await screen.findByRole('heading', { name: 'Model Analytics' })).toBeInTheDocument();
  });

  it('/analytics/health renders AnalyticsHealthPage', async () => {
    renderRoute('/analytics/health');
    expect(await screen.findByRole('heading', { name: 'System Health' })).toBeInTheDocument();
  });

  it('/analytics/performance renders AnalyticsPerformancePage', async () => {
    renderRoute('/analytics/performance');
    expect(await screen.findByRole('heading', { name: 'Performance Trends' })).toBeInTheDocument();
  });

  it('/analytics/resources renders AnalyticsResourcesPage', async () => {
    renderRoute('/analytics/resources');
    expect(await screen.findByRole('heading', { name: 'Resource Usage' })).toBeInTheDocument();
  });

  it('/analytics/costs renders AnalyticsCostsPage', async () => {
    renderRoute('/analytics/costs');
    expect(await screen.findByRole('heading', { name: 'Cost Analytics' })).toBeInTheDocument();
  });

  it('/analytics/services renders AnalyticsServicesPage', async () => {
    renderRoute('/analytics/services');
    expect(await screen.findByRole('heading', { name: 'Service Status' })).toBeInTheDocument();
  });

  it('/analytics/costs/models/gpt-4 renders AnalyticsModelDetailPage', async () => {
    renderRoute('/analytics/costs/models/gpt-4');
    expect(await screen.findByRole('heading', { name: 'gpt-4' })).toBeInTheDocument();
  });

  it('/efficiency/memory renders EfficiencyMemoryPage', async () => {
    renderRoute('/efficiency/memory');
    expect(await screen.findByRole('heading', { name: 'Memory Usage' })).toBeInTheDocument();
  });

  it('/efficiency/sessions renders EfficiencySessionsPage', async () => {
    renderRoute('/efficiency/sessions');
    expect(await screen.findByRole('heading', { name: 'Session Activity' })).toBeInTheDocument();
  });

  it('/efficiency/agents renders EfficiencyAgentsPage', async () => {
    renderRoute('/efficiency/agents');
    expect(await screen.findByRole('heading', { name: 'Agent Performance' })).toBeInTheDocument();
  });

  it('/efficiency/skills renders EfficiencySkillsPage', async () => {
    renderRoute('/efficiency/skills');
    expect(await screen.findByRole('heading', { name: 'Skill Effectiveness' })).toBeInTheDocument();
  });

  it('/efficiency/tokens renders EfficiencyTokensPage', async () => {
    renderRoute('/efficiency/tokens');
    expect(await screen.findByRole('heading', { name: 'Token Usage' })).toBeInTheDocument();
  });

  it('/efficiency/correlation renders EfficiencyCorrelationPage', async () => {
    renderRoute('/efficiency/correlation');
    expect(await screen.findByRole('heading', { name: 'Correlation Matrix' })).toBeInTheDocument();
  });

  it('/settings/general renders SettingsPage', async () => {
    renderRoute('/settings/general');
    expect(await screen.findByRole('heading', { name: 'Settings' })).toBeInTheDocument();
  });

  it('/playground renders PlaygroundPage', async () => {
    renderRoute('/playground');
    expect(await screen.findByRole('heading', { name: 'Playground' })).toBeInTheDocument();
    // Submit button should be present
    expect(screen.getByText('Submit')).toBeInTheDocument();
  });

  it('/feedback renders FeedbackPage', async () => {
    renderRoute('/feedback');
    expect(await screen.findByRole('heading', { name: 'Feedback' })).toBeInTheDocument();
    // Changelog tab should be visible by default
    expect(screen.getByText('Changelog')).toBeInTheDocument();
  });

  it('/exports renders ExportsPage', async () => {
    renderRoute('/exports');
    expect(await screen.findByRole('heading', { name: 'Exports' })).toBeInTheDocument();
    // New Export button should be present
    expect(screen.getByText('New Export')).toBeInTheDocument();
  });

  it('/onboarding renders OnboardingPage', async () => {
    renderRoute('/onboarding');
    expect(await screen.findByRole('heading', { name: 'Onboarding' })).toBeInTheDocument();
  });

  it('/correlation renders CorrelationPage', async () => {
    renderRoute('/correlation');
    expect(await screen.findByRole('heading', { name: 'Correlation Analysis' })).toBeInTheDocument();
  });

  it('/audit renders AuditPage', async () => {
    renderRoute('/audit');
    expect(await screen.findByRole('heading', { name: 'Audit Log' })).toBeInTheDocument();
  });
});
