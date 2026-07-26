import { http, HttpResponse } from 'msw';
import type { HttpHandler } from 'msw';

export const correlationHandlers: HttpHandler[] = [
  // GET /api/v1/correlation/overview
  http.get('*/api/v1/correlation/overview', () => {
    return HttpResponse.json({
      dataset_stats: [
        { variable: 'efficiency_score', mean: 78.5, std: 12.3, min: 45, max: 98 },
        { variable: 'tokens_per_session', mean: 1250, std: 450, min: 200, max: 4000 },
        { variable: 'session_duration', mean: 45, std: 20, min: 5, max: 180 },
        { variable: 'memory_count', mean: 35, std: 15, min: 0, max: 120 },
      ],
      top_correlations: [
        { variable_1: 'efficiency_score', variable_2: 'session_duration', r: -0.45, p_value: 0.001 },
        { variable_1: 'tokens_per_session', variable_2: 'memory_count', r: 0.62, p_value: 0.0001 },
        { variable_1: 'session_duration', variable_2: 'memory_count', r: 0.38, p_value: 0.01 },
      ],
    });
  }),

  // GET /api/v1/correlation/timeline
  http.get('*/api/v1/correlation/timeline', () => {
    return HttpResponse.json([
      {
        date: '2026-07-20',
        correlations: [
          { variable_1: 'efficiency', variable_2: 'tokens', r: -0.42 },
          { variable_1: 'efficiency', variable_2: 'sessions', r: 0.58 },
        ],
      },
      {
        date: '2026-07-21',
        correlations: [
          { variable_1: 'efficiency', variable_2: 'tokens', r: -0.38 },
          { variable_1: 'efficiency', variable_2: 'sessions', r: 0.62 },
        ],
      },
      {
        date: '2026-07-22',
        correlations: [
          { variable_1: 'efficiency', variable_2: 'tokens', r: -0.45 },
          { variable_1: 'efficiency', variable_2: 'sessions', r: 0.55 },
        ],
      },
    ]);
  }),

  // GET /api/v1/correlation/compare
  http.get('*/api/v1/correlation/compare', () => {
    return HttpResponse.json({
      groups: ['gpt-4', 'gpt-3.5', 'claude-3'],
      metric: 'efficiency_score',
      values: [
        { group: 'gpt-4', mean: 85.2, std: 8.5, n: 45 },
        { group: 'gpt-3.5', mean: 72.8, std: 12.1, n: 38 },
        { group: 'claude-3', mean: 80.5, std: 10.2, n: 22 },
      ],
      test: {
        type: 'anova',
        statistic: 12.45,
        p_value: 0.0001,
        significant: true,
      },
    });
  }),
];
