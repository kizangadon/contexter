import { http, HttpResponse } from 'msw';
import type { HttpHandler } from 'msw';
import type { AgentDetail } from '@/api/types';
import { buildAgentDetail } from '../factories/agentFactory';

const agents = new Map<string, AgentDetail>();

function seedAgents(): void {
  const ids = ['agt_000001', 'agt_000002'];
  for (const id of ids) {
    const detail = buildAgentDetail({ id });
    agents.set(id, detail);
  }
}
seedAgents();

export const agentsHandlers: HttpHandler[] = [
  // GET /api/v1/agents — list with optional status filter
  http.get('*/api/v1/agents', ({ request }) => {
    const url = new URL(request.url);
    const status = url.searchParams.get('status');

    let list = Array.from(agents.values());
    if (status) {
      list = list.filter((a) => a.status === status);
    }
    return HttpResponse.json(list);
  }),

  // GET /api/v1/agents/:id — agent detail
  http.get('*/api/v1/agents/:id', ({ params }) => {
    const agent = agents.get(params.id as string);
    if (!agent) {
      return HttpResponse.json({ detail: 'Agent not found' }, { status: 404 });
    }
    return HttpResponse.json(agent);
  }),

  // POST /api/v1/agents — create
  http.post('*/api/v1/agents', async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown> | undefined;
    const agent = buildAgentDetail({
      name: (body?.name as string) ?? 'New Agent',
      capabilities: (body?.capabilities as string[]) ?? [],
    });
    agents.set(agent.id, agent);
    return HttpResponse.json(agent, { status: 201 });
  }),
];
