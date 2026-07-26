import type { Agent, AgentDetail, Session } from '@/api/types';
import { buildSession } from './sessionFactory';

let agentCounter = 0;

export function resetAgentCounters(): void {
  agentCounter = 0;
}

export function buildAgent(overrides?: Partial<Agent>): Agent {
  agentCounter += 1;

  const id = `agt_${String(agentCounter).padStart(6, '0')}`;
  const now = new Date('2026-07-26T00:00:00Z');
  const created = new Date(now.getTime() - 604800000 * agentCounter);
  const lastActive = new Date(now.getTime() - 60000 * (agentCounter - 1));

  return {
    id,
    name: `Agent-${agentCounter}`,
    capabilities: ['code-review', 'debugging', 'refactoring'],
    status: 'active',
    efficiency_score: 85,
    sessions_count: 42,
    avg_latency_ms: 320,
    created_at: created.toISOString(),
    last_active: lastActive.toISOString(),
    ...overrides,
  };
}

export function buildAgentList(count = 3): Agent[] {
  resetAgentCounters();
  return Array.from({ length: count }, () => buildAgent());
}

export function buildAgentDetail(overrides?: Partial<AgentDetail>): AgentDetail {
  const agent = buildAgent(overrides);
  const sessions: Session[] = [
    buildSession({ agent: agent.name, status: 'done' }),
    buildSession({ agent: agent.name, status: 'active' }),
  ];

  return {
    ...agent,
    recent_sessions: sessions,
    efficiency_history: [
      { date: '2026-07-19', score: 82 },
      { date: '2026-07-20', score: 84 },
      { date: '2026-07-21', score: 85 },
    ],
    settings: { max_tokens: 4096, temperature: 0.7 },
    ...overrides,
  } as AgentDetail;
}
