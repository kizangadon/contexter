import { http, HttpResponse } from 'msw';
import type { HttpHandler } from 'msw';

export const analyticsHandlers: HttpHandler[] = [
  // GET /api/v1/analytics/overview
  http.get('*/api/v1/analytics/overview', () => {
    return HttpResponse.json({
      system_health: 'healthy',
      uptime_percent: 99.95,
      error_rate: 0.02,
      avg_response_time_ms: 245,
      active_sessions: 12,
      memory_usage_percent: 58,
      api_requests_total: 45230,
      cost_total: 342.5,
    });
  }),

  // GET /api/v1/analytics/health
  http.get('*/api/v1/analytics/health', () => {
    return HttpResponse.json({
      status: 'healthy',
      uptime_seconds: 864000,
      version: '1.0.0',
      services: {
        api: 'healthy',
        database: 'healthy',
        mcp: 'healthy',
      },
    });
  }),

  // GET /api/v1/analytics/performance
  http.get('*/api/v1/analytics/performance', () => {
    return HttpResponse.json([
      { date: '2026-07-20', response_time_ms: 250, throughput: 120, error_rate: 0.03 },
      { date: '2026-07-21', response_time_ms: 240, throughput: 135, error_rate: 0.02 },
      { date: '2026-07-22', response_time_ms: 235, throughput: 150, error_rate: 0.01 },
      { date: '2026-07-23', response_time_ms: 260, throughput: 110, error_rate: 0.04 },
      { date: '2026-07-24', response_time_ms: 230, throughput: 160, error_rate: 0.02 },
      { date: '2026-07-25', response_time_ms: 245, throughput: 145, error_rate: 0.02 },
      { date: '2026-07-26', response_time_ms: 255, throughput: 130, error_rate: 0.03 },
    ]);
  }),

  // GET /api/v1/analytics/resources
  http.get('*/api/v1/analytics/resources', () => {
    return HttpResponse.json({
      cpu_percent: 45,
      memory_percent: 62,
      disk_percent: 38,
      active_connections: 28,
    });
  }),

  // GET /api/v1/analytics/costs
  http.get('*/api/v1/analytics/costs', () => {
    return HttpResponse.json({
      total_cost: 342.5,
      by_model: [
        { model: 'gpt-4', cost: 200.0, tokens: 80000, percentage: 58.4 },
        { model: 'gpt-3.5', cost: 92.5, tokens: 58000, percentage: 27.0 },
        { model: 'claude-3', cost: 50.0, tokens: 20000, percentage: 14.6 },
      ],
      daily_costs: [
        { date: '2026-07-20', cost: 48.5 },
        { date: '2026-07-21', cost: 52.0 },
        { date: '2026-07-22', cost: 45.0 },
        { date: '2026-07-23', cost: 55.5 },
        { date: '2026-07-24', cost: 42.0 },
        { date: '2026-07-25', cost: 50.5 },
        { date: '2026-07-26', cost: 49.0 },
      ],
    });
  }),

  // GET /api/v1/analytics/costs/:model — model detail
  http.get('*/api/v1/analytics/costs/:model', ({ params }) => {
    const modelMap: Record<string, unknown> = {
      'gpt-4': {
        model: 'gpt-4',
        total_cost: 200.0,
        total_tokens: 80000,
        input_tokens: 60000,
        output_tokens: 20000,
        avg_cost_per_token: 0.0025,
        daily_breakdown: [
          { date: '2026-07-20', tokens: 12000, cost: 30.0 },
          { date: '2026-07-21', tokens: 11000, cost: 27.5 },
          { date: '2026-07-22', tokens: 10500, cost: 26.25 },
          { date: '2026-07-23', tokens: 11500, cost: 28.75 },
          { date: '2026-07-24', tokens: 9800, cost: 24.5 },
          { date: '2026-07-25', tokens: 10200, cost: 25.5 },
          { date: '2026-07-26', tokens: 15000, cost: 37.5 },
        ],
      },
      'gpt-3.5': {
        model: 'gpt-3.5',
        total_cost: 92.5,
        total_tokens: 58000,
        input_tokens: 45000,
        output_tokens: 13000,
        avg_cost_per_token: 0.0016,
        daily_breakdown: [
          { date: '2026-07-20', tokens: 8000, cost: 12.8 },
          { date: '2026-07-21', tokens: 7500, cost: 12.0 },
          { date: '2026-07-22', tokens: 7000, cost: 11.2 },
          { date: '2026-07-23', tokens: 9000, cost: 14.4 },
          { date: '2026-07-24', tokens: 6500, cost: 10.4 },
          { date: '2026-07-25', tokens: 10000, cost: 16.0 },
          { date: '2026-07-26', tokens: 10000, cost: 16.0 },
        ],
      },
    };

    const data = modelMap[params.model as string];
    if (!data) {
      return HttpResponse.json({ detail: 'Model not found' }, { status: 404 });
    }
    return HttpResponse.json(data);
  }),

  // GET /api/v1/analytics/services
  http.get('*/api/v1/analytics/services', () => {
    return HttpResponse.json([
      { name: 'API Gateway', status: 'healthy', uptime_percent: 99.99, latency_ms: 12, last_checked: '2026-07-26T00:00:00Z' },
      { name: 'Database', status: 'healthy', uptime_percent: 99.95, latency_ms: 5, last_checked: '2026-07-26T00:00:00Z' },
      { name: 'MCP Server', status: 'healthy', uptime_percent: 99.90, latency_ms: 25, last_checked: '2026-07-26T00:00:00Z' },
      { name: 'Redis Cache', status: 'healthy', uptime_percent: 99.98, latency_ms: 2, last_checked: '2026-07-26T00:00:00Z' },
      { name: 'Vector Store', status: 'degraded', uptime_percent: 98.50, latency_ms: 150, last_checked: '2026-07-26T00:00:00Z' },
    ]);
  }),
];
