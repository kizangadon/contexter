import { http, HttpResponse } from 'msw';
import type { HttpHandler } from 'msw';
import type { BugReport, FeatureRequest, ChangelogEntry } from '@/api/types';

const bugs: BugReport[] = [
  { id: 'bug_000001', title: 'Session list not paginating', description: 'When sessions exceed 50, the list does not show pagination controls.', severity: 'medium', status: 'open', created_at: '2026-07-24T10:00:00Z' },
  { id: 'bug_000002', title: 'Memory search returns stale results', description: 'After updating a memory, search still returns old content for up to 5 minutes.', severity: 'high', status: 'in-progress', created_at: '2026-07-23T14:30:00Z' },
];

const suggestions: FeatureRequest[] = [
  { id: 'fr_000001', title: 'Dark mode toggle', description: 'Add a toggle to switch between dark and light themes.', status: 'under-review', votes: 15, created_at: '2026-07-20T08:00:00Z' },
  { id: 'fr_000002', title: 'Export to PDF', description: 'Allow exporting session reports as PDF files.', status: 'planned', votes: 8, created_at: '2026-07-21T12:00:00Z' },
];

const changelog: ChangelogEntry[] = [
  {
    version: '1.1.0',
    date: '2026-07-25',
    changes: [
      { type: 'added', description: 'Memory search with full-text indexing' },
      { type: 'changed', description: 'Improved session loading performance' },
      { type: 'fixed', description: 'Agent status not updating in real-time' },
    ],
  },
  {
    version: '1.0.0',
    date: '2026-07-15',
    changes: [
      { type: 'added', description: 'Initial release with core features' },
      { type: 'added', description: 'Session management and monitoring' },
    ],
  },
];

export const feedbackHandlers: HttpHandler[] = [
  // GET /api/v1/feedback/bugs
  http.get('*/api/v1/feedback/bugs', () => {
    return HttpResponse.json(bugs);
  }),

  // POST /api/v1/feedback/bugs — submit bug report
  http.post('*/api/v1/feedback/bugs', async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown>;
    const newBug: BugReport = {
      id: `bug_${String(bugs.length + 1).padStart(6, '0')}`,
      title: (body.title as string) ?? 'Untitled',
      description: (body.description as string) ?? '',
      severity: (body.severity as BugReport['severity']) ?? 'low',
      status: 'open',
      created_at: new Date().toISOString(),
    };
    bugs.push(newBug);
    return HttpResponse.json(newBug, { status: 201 });
  }),

  // GET /api/v1/feedback/suggestions
  http.get('*/api/v1/feedback/suggestions', () => {
    return HttpResponse.json(suggestions);
  }),

  // POST /api/v1/feedback/suggestions — submit suggestion
  http.post('*/api/v1/feedback/suggestions', async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown>;
    const newSuggestion: FeatureRequest = {
      id: `fr_${String(suggestions.length + 1).padStart(6, '0')}`,
      title: (body.title as string) ?? 'Untitled',
      description: (body.description as string) ?? '',
      status: 'under-review',
      votes: 0,
      created_at: new Date().toISOString(),
    };
    suggestions.push(newSuggestion);
    return HttpResponse.json(newSuggestion, { status: 201 });
  }),

  // GET /api/v1/feedback/changelog
  http.get('*/api/v1/feedback/changelog', () => {
    return HttpResponse.json(changelog);
  }),
];
