import { http, HttpResponse } from 'msw';
import type { HttpHandler } from 'msw';

const searchResults = [
  { id: 'ses_000001', type: 'session' as const, title: 'Code Review Session', snippet: 'Reviewed pull request #42 for the API layer...', score: 0.95 },
  { id: 'mem_000001', type: 'memory' as const, title: 'Architecture Insight', snippet: 'The system uses a layered architecture with clear separation...', score: 0.88 },
  { id: 'agt_000001', type: 'agent' as const, title: 'Agent-1', snippet: 'Specialized in code review and debugging tasks...', score: 0.72 },
  { id: 'skl_000001', type: 'skill' as const, title: 'Review Pro', snippet: 'Expert-level code review with comprehensive analysis...', score: 0.65 },
];

export const searchHandlers: HttpHandler[] = [
  // GET /api/v1/search?q=
  http.get('*/api/v1/search', ({ request }) => {
    const url = new URL(request.url);
    const query = url.searchParams.get('q')?.toLowerCase() ?? '';

    if (!query || query.length < 2) {
      return HttpResponse.json([]);
    }

    const results = searchResults.filter(
      (r) =>
        r.title.toLowerCase().includes(query) ||
        r.snippet.toLowerCase().includes(query) ||
        r.type.includes(query),
    );

    return HttpResponse.json(results);
  }),
];
