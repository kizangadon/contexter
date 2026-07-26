import { http, HttpResponse } from 'msw';
import type { HttpHandler } from 'msw';
import type { SessionDetail } from '@/api/types';
import { buildSessionDetail } from '../factories/sessionFactory';

// In-memory store seeded with a few entries
const sessions = new Map<string, SessionDetail>();

function seedSessions(): void {
  const ids = ['ses_000001', 'ses_000002', 'ses_000003'];
  for (const id of ids) {
    const detail = buildSessionDetail({ id });
    sessions.set(id, detail);
  }
}
seedSessions();

export const sessionsHandlers: HttpHandler[] = [
  // GET /api/v1/sessions — list with optional status/project filter
  http.get('*/api/v1/sessions', ({ request }) => {
    const url = new URL(request.url);
    const status = url.searchParams.get('status');
    const project = url.searchParams.get('project');

    let list = Array.from(sessions.values());

    if (status) {
      list = list.filter((s) => s.status === status);
    }
    if (project) {
      list = list.filter((s) => s.project === project);
    }

    return HttpResponse.json(list);
  }),

  // GET /api/v1/sessions/:id — session detail
  http.get('*/api/v1/sessions/:id', ({ params }) => {
    const session = sessions.get(params.id as string);
    if (!session) {
      return HttpResponse.json({ detail: 'Session not found' }, { status: 404 });
    }
    return HttpResponse.json(session);
  }),

  // POST /api/v1/sessions — create
  http.post('*/api/v1/sessions', async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown> | undefined;
    const session = buildSessionDetail({
      project: (body?.project as string) ?? 'contexter',
      agent: (body?.agent as string) ?? 'default-agent',
    });
    sessions.set(session.id, session);
    return HttpResponse.json(session, { status: 201 });
  }),

  // PATCH /api/v1/sessions/:id — update
  http.patch('*/api/v1/sessions/:id', async ({ params, request }) => {
    const existing = sessions.get(params.id as string);
    if (!existing) {
      return HttpResponse.json({ detail: 'Session not found' }, { status: 404 });
    }
    const body = (await request.json()) as Record<string, unknown>;
    const updated = { ...existing, ...body } as SessionDetail;
    sessions.set(params.id as string, updated);
    return HttpResponse.json(updated);
  }),

  // DELETE /api/v1/sessions/:id — delete
  http.delete('*/api/v1/sessions/:id', ({ params }) => {
    if (!sessions.has(params.id as string)) {
      return HttpResponse.json({ detail: 'Session not found' }, { status: 404 });
    }
    sessions.delete(params.id as string);
    return HttpResponse.json(null, { status: 204 });
  }),

  // POST /api/v1/sessions/:id/resume — resume a session
  http.post('*/api/v1/sessions/:id/resume', ({ params }) => {
    const existing = sessions.get(params.id as string);
    if (!existing) {
      return HttpResponse.json({ detail: 'Session not found' }, { status: 404 });
    }
    const updated: SessionDetail = {
      ...existing,
      status: 'active',
      last_active: new Date().toISOString(),
    };
    sessions.set(params.id as string, updated);
    return HttpResponse.json(updated);
  }),
];
