/* oxlint-disable react/only-export-components — lazy component variables for route definitions */

import { lazy } from 'react';
import type { RouteObject } from 'react-router';

const DashboardPage = lazy(() => import('./pages/Dashboard/DashboardPage').then(m => ({ default: m.DashboardPage })));
const AgentRegistryPage = lazy(() => import('./pages/Agents/AgentRegistryPage').then(m => ({ default: m.AgentRegistryPage })));
const AgentDetailPage = lazy(() => import('./pages/Agents/AgentDetailPage').then(m => ({ default: m.AgentDetailPage })));
const MemoryExplorerPage = lazy(() => import('./pages/Memories/MemoryExplorerPage').then(m => ({ default: m.MemoryExplorerPage })));
const MemoryDetailPage = lazy(() => import('./pages/Memories/MemoryDetailPage').then(m => ({ default: m.MemoryDetailPage })));
const SkillRegistryPage = lazy(() => import('./pages/Skills/SkillRegistryPage').then(m => ({ default: m.SkillRegistryPage })));
const SkillDetailPage = lazy(() => import('./pages/Skills/SkillDetailPage').then(m => ({ default: m.SkillDetailPage })));
const SessionManagerPage = lazy(() => import('./pages/Sessions/SessionManagerPage').then(m => ({ default: m.SessionManagerPage })));
const SessionDetailPage = lazy(() => import('./pages/Sessions/SessionDetailPage').then(m => ({ default: m.SessionDetailPage })));
const SettingsPage = lazy(() => import('./pages/Settings/SettingsPage').then(m => ({ default: m.SettingsPage })));
const EfficiencyPage = lazy(() => import('./pages/Efficiency/EfficiencyPage').then(m => ({ default: m.EfficiencyPage })));
const SearchPage = lazy(() => import('./pages/Search/SearchPage').then(m => ({ default: m.SearchPage })));
const PlaygroundPage = lazy(() => import('./pages/Playground/PlaygroundPage').then(m => ({ default: m.PlaygroundPage })));
const NotificationsPage = lazy(() => import('./pages/Notifications/NotificationsPage').then(m => ({ default: m.NotificationsPage })));
const FeedbackPage = lazy(() => import('./pages/Feedback/FeedbackPage').then(m => ({ default: m.FeedbackPage })));
const ExportsPage = lazy(() => import('./pages/Exports/ExportsPage').then(m => ({ default: m.ExportsPage })));
const OnboardingPage = lazy(() => import('./pages/Onboarding/OnboardingPage').then(m => ({ default: m.OnboardingPage })));
const CorrelationPage = lazy(() => import('./pages/Correlation/CorrelationPage').then(m => ({ default: m.CorrelationPage })));
const AnalyticsDashboardPage = lazy(() => import('./pages/Analytics/AnalyticsDashboardPage').then(m => ({ default: m.AnalyticsDashboardPage })));
const AuditPage = lazy(() => import('./pages/Audit/AuditPage').then(m => ({ default: m.AuditPage })));
const NotFoundPage = lazy(() => import('./pages/NotFound/NotFoundPage').then(m => ({ default: m.NotFoundPage })));
const AnalyticsModelsPage = lazy(() => import('./pages/Analytics/AnalyticsModelsPage').then(m => ({ default: m.AnalyticsModelsPage })));
const AnalyticsHealthPage = lazy(() => import('./pages/Analytics/AnalyticsHealthPage').then(m => ({ default: m.AnalyticsHealthPage })));
const AnalyticsPerformancePage = lazy(() => import('./pages/Analytics/AnalyticsPerformancePage').then(m => ({ default: m.AnalyticsPerformancePage })));
const AnalyticsResourcesPage = lazy(() => import('./pages/Analytics/AnalyticsResourcesPage').then(m => ({ default: m.AnalyticsResourcesPage })));
const AnalyticsCostsPage = lazy(() => import('./pages/Analytics/AnalyticsCostsPage').then(m => ({ default: m.AnalyticsCostsPage })));
const AnalyticsModelDetailPage = lazy(() => import('./pages/Analytics/AnalyticsModelDetailPage').then(m => ({ default: m.AnalyticsModelDetailPage })));
const AnalyticsServicesPage = lazy(() => import('./pages/Analytics/AnalyticsServicesPage').then(m => ({ default: m.AnalyticsServicesPage })));
const EfficiencyMemoryPage = lazy(() => import('./pages/Efficiency/EfficiencyMemoryPage').then(m => ({ default: m.EfficiencyMemoryPage })));
const EfficiencySessionsPage = lazy(() => import('./pages/Efficiency/EfficiencySessionsPage').then(m => ({ default: m.EfficiencySessionsPage })));
const EfficiencyAgentsPage = lazy(() => import('./pages/Efficiency/EfficiencyAgentsPage').then(m => ({ default: m.EfficiencyAgentsPage })));
const EfficiencySkillsPage = lazy(() => import('./pages/Efficiency/EfficiencySkillsPage').then(m => ({ default: m.EfficiencySkillsPage })));
const EfficiencyTokensPage = lazy(() => import('./pages/Efficiency/EfficiencyTokensPage').then(m => ({ default: m.EfficiencyTokensPage })));
const EfficiencyCorrelationPage = lazy(() => import('./pages/Efficiency/EfficiencyCorrelationPage').then(m => ({ default: m.EfficiencyCorrelationPage })));

export const routes: RouteObject[] = [
  /* ── Dashboard ─────────────────────────────────────────── */
  { path: '/dashboard', element: <DashboardPage /> },

  /* ── Sessions ──────────────────────────────────────────── */
  { path: '/sessions', element: <SessionManagerPage /> },
  { path: '/sessions/:id', element: <SessionDetailPage /> },

  /* ── Memories ──────────────────────────────────────────── */
  { path: '/memories', element: <MemoryExplorerPage /> },
  { path: '/memories/:id', element: <MemoryDetailPage /> },

  /* ── Agents ────────────────────────────────────────────── */
  { path: '/agents', element: <AgentRegistryPage /> },
  { path: '/agents/:id', element: <AgentDetailPage /> },

  /* ── Skills ────────────────────────────────────────────── */
  { path: '/skills', element: <SkillRegistryPage /> },
  { path: '/skills/:id', element: <SkillDetailPage /> },

  /* ── Efficiency (main + 6 sub-pages) ───────────────────── */
  { path: '/efficiency', element: <EfficiencyPage /> },
  { path: '/efficiency/memory', element: <EfficiencyMemoryPage /> },
  { path: '/efficiency/sessions', element: <EfficiencySessionsPage /> },
  { path: '/efficiency/agents', element: <EfficiencyAgentsPage /> },
  { path: '/efficiency/skills', element: <EfficiencySkillsPage /> },
  { path: '/efficiency/tokens', element: <EfficiencyTokensPage /> },
  { path: '/efficiency/correlation', element: <EfficiencyCorrelationPage /> },

  /* ── Analytics (main + 6 sub-pages) ────────────────────── */
  { path: '/analytics', element: <AnalyticsDashboardPage /> },
  { path: '/analytics/health', element: <AnalyticsHealthPage /> },
  { path: '/analytics/performance', element: <AnalyticsPerformancePage /> },
  { path: '/analytics/resources', element: <AnalyticsResourcesPage /> },
  { path: '/analytics/costs', element: <AnalyticsCostsPage /> },
  { path: '/analytics/costs/models/:id', element: <AnalyticsModelDetailPage /> },
  { path: '/analytics/models', element: <AnalyticsModelsPage /> },
  { path: '/analytics/services', element: <AnalyticsServicesPage /> },

  /* ── Settings (8 sections via :section param) ──────────── */
  { path: '/settings', element: <SettingsPage /> },
  { path: '/settings/:section', element: <SettingsPage /> },

  /* ── Standalone pages ──────────────────────────────────── */
  { path: '/search', element: <SearchPage /> },
  { path: '/playground', element: <PlaygroundPage /> },
  { path: '/notifications', element: <NotificationsPage /> },
  { path: '/feedback', element: <FeedbackPage /> },
  { path: '/exports', element: <ExportsPage /> },
  { path: '/onboarding', element: <OnboardingPage /> },
  { path: '/correlation', element: <CorrelationPage /> },
  { path: '/audit', element: <AuditPage /> },

  /* ── Fallback ──────────────────────────────────────────── */
  { path: '*', element: <NotFoundPage /> },
];
