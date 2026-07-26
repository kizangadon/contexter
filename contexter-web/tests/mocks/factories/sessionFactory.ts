import type { Session, SessionDetail, Turn } from '@/api/types';

let sessionCounter = 0;
let turnCounter = 0;

export function resetSessionCounters(): void {
  sessionCounter = 0;
  turnCounter = 0;
}

export function buildSession(overrides?: Partial<Session>): Session {
  sessionCounter += 1;

  const id = `ses_${String(sessionCounter).padStart(6, '0')}`;
  const now = new Date('2026-07-26T00:00:00Z');
  const created = new Date(now.getTime() - 3600000 * sessionCounter);
  const lastActive = new Date(created.getTime() + 1800000);

  return {
    id,
    project: 'contexter',
    agent: 'default-agent',
    status: 'active',
    duration_minutes: 30,
    turn_count: 5,
    created_at: created.toISOString(),
    last_active: lastActive.toISOString(),
    ...overrides,
  };
}

export function buildSessionList(count = 3): Session[] {
  resetSessionCounters();
  return Array.from({ length: count }, () => buildSession());
}

export function buildTurn(overrides?: Partial<Turn>): Turn {
  turnCounter += 1;

  const now = new Date('2026-07-26T00:00:00Z');
  const created = new Date(now.getTime() - 60000 * turnCounter);

  return {
    id: `trn_${String(turnCounter).padStart(6, '0')}`,
    session_id: `ses_000001`,
    number: turnCounter,
    role: turnCounter % 2 === 1 ? 'user' : 'agent',
    content: 'Sample turn content for testing.',
    latency_ms: 150,
    created_at: created.toISOString(),
    ...overrides,
  };
}

export function buildSessionDetail(overrides?: Partial<SessionDetail>): SessionDetail {
  const session = buildSession(overrides);
  return {
    ...session,
    turns: [buildTurn({ session_id: session.id }), buildTurn({ session_id: session.id })],
    memories_created: 3,
    tokens_used: 1500,
    tags: ['exploration', 'debugging'],
    ...overrides,
  } as SessionDetail;
}
