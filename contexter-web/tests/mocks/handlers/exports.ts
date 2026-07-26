import { http, HttpResponse } from 'msw';
import type { HttpHandler } from 'msw';
import type { ExportJob } from '@/api/types';

const exportsList: ExportJob[] = [
  { id: 'exp_000001', type: 'sessions', format: 'json', status: 'completed', created_at: '2026-07-25T12:00:00Z', completed_at: '2026-07-25T12:00:05Z', download_url: '/api/v1/exports/exp_000001/download' },
  { id: 'exp_000002', type: 'analytics', format: 'csv', status: 'processing', created_at: '2026-07-26T00:00:00Z' },
  { id: 'exp_000003', type: 'memories', format: 'json', status: 'failed', created_at: '2026-07-24T08:00:00Z', completed_at: '2026-07-24T08:00:03Z', error: 'Insufficient memory data' },
];

export const exportsHandlers: HttpHandler[] = [
  // GET /api/v1/exports
  http.get('*/api/v1/exports', () => {
    return HttpResponse.json(exportsList);
  }),

  // POST /api/v1/exports — submit new export
  http.post('*/api/v1/exports', async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown>;
    const newExport: ExportJob = {
      id: `exp_${String(exportsList.length + 1).padStart(6, '0')}`,
      type: (body.type as ExportJob['type']) ?? 'sessions',
      format: (body.format as ExportJob['format']) ?? 'json',
      status: 'pending',
      created_at: new Date().toISOString(),
    };
    exportsList.push(newExport);
    return HttpResponse.json(newExport, { status: 201 });
  }),
];
