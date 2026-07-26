import { http, HttpResponse } from 'msw';
import type { HttpHandler } from 'msw';

export const efficiencyHandlers: HttpHandler[] = [
  // GET /api/v1/efficiency/overview
  http.get('*/api/v1/efficiency/overview', () => {
    return HttpResponse.json({
      avg_efficiency: 78.5,
      trend: 5.2,
      avg_tokens: 1250,
      avg_duration_minutes: 45,
      memory_used_percent: 62,
      session_count: 128,
      agent_count: 4,
      skill_count: 12,
    });
  }),

  // GET /api/v1/efficiency/memory
  http.get('*/api/v1/efficiency/memory', () => {
    return HttpResponse.json({
      total_memories: 450,
      avg_confidence: 0.82,
      type_distribution: {
        conversation: 200,
        decision: 100,
        pattern: 80,
        reference: 50,
        custom: 20,
      },
    });
  }),

  // GET /api/v1/efficiency/sessions
  http.get('*/api/v1/efficiency/sessions', () => {
    return HttpResponse.json([
      { date: '2026-07-20', score: 75, tokens: 1200, sessions: 8 },
      { date: '2026-07-21', score: 78, tokens: 1100, sessions: 10 },
      { date: '2026-07-22', score: 82, tokens: 1050, sessions: 12 },
      { date: '2026-07-23', score: 80, tokens: 1150, sessions: 9 },
      { date: '2026-07-24', score: 85, tokens: 980, sessions: 11 },
      { date: '2026-07-25', score: 83, tokens: 1020, sessions: 13 },
      { date: '2026-07-26', score: 78, tokens: 1250, sessions: 7 },
    ]);
  }),

  // GET /api/v1/efficiency/agents
  http.get('*/api/v1/efficiency/agents', () => {
    return HttpResponse.json([
      { agent_id: 'agt_000001', agent_name: 'Agent-1', efficiency_score: 88, sessions_count: 52, avg_latency_ms: 280, trend: 3.5 },
      { agent_id: 'agt_000002', agent_name: 'Agent-2', efficiency_score: 75, sessions_count: 38, avg_latency_ms: 410, trend: -1.2 },
      { agent_id: 'agt_000003', agent_name: 'Agent-3', efficiency_score: 82, sessions_count: 45, avg_latency_ms: 340, trend: 2.1 },
    ]);
  }),

  // GET /api/v1/efficiency/skills
  http.get('*/api/v1/efficiency/skills', () => {
    return HttpResponse.json([
      { skill_id: 'skl_000001', skill_name: 'Review Pro', effectiveness_score: 90, usage_count: 85, trend: 4.5 },
      { skill_id: 'skl_000002', skill_name: 'Bug Hunter', effectiveness_score: 78, usage_count: 62, trend: -0.5 },
      { skill_id: 'skl_000003', skill_name: 'Refactor Master', effectiveness_score: 85, usage_count: 45, trend: 6.2 },
    ]);
  }),

  // GET /api/v1/efficiency/tokens
  http.get('*/api/v1/efficiency/tokens', () => {
    return HttpResponse.json({
      total_tokens: 158000,
      avg_per_session: 1234,
      by_model: {
        'gpt-4': 80000,
        'gpt-3.5': 58000,
        claude: 20000,
      },
      daily: [
        { date: '2026-07-20', tokens: 12000 },
        { date: '2026-07-21', tokens: 11000 },
        { date: '2026-07-22', tokens: 10500 },
        { date: '2026-07-23', tokens: 11500 },
        { date: '2026-07-24', tokens: 9800 },
        { date: '2026-07-25', tokens: 10200 },
        { date: '2026-07-26', tokens: 12500 },
      ],
    });
  }),

  // GET /api/v1/efficiency/correlation
  http.get('*/api/v1/efficiency/correlation', () => {
    return HttpResponse.json({
      variables: ['efficiency', 'tokens', 'sessions', 'latency'],
      correlations: [
        [1.0, -0.45, 0.62, -0.38],
        [-0.45, 1.0, -0.22, 0.51],
        [0.62, -0.22, 1.0, -0.55],
        [-0.38, 0.51, -0.55, 1.0],
      ],
    });
  }),
];
