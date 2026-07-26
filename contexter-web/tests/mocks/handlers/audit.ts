import { http, HttpResponse } from 'msw';
import type { HttpHandler } from 'msw';
import type { AuditEntry } from '@/api/types';

const auditLog: AuditEntry[] = [
  {
    id: 'aud_000001',
    action: 'session.create',
    entity_type: 'session',
    entity_id: 'ses_000001',
    changes: [{ field: 'status', old_value: undefined, new_value: 'active' }],
    performed_by: 'user-001',
    created_at: '2026-07-26T00:00:00Z',
  },
  {
    id: 'aud_000002',
    action: 'memory.update',
    entity_type: 'memory',
    entity_id: 'mem_000001',
    changes: [
      { field: 'content', old_value: 'Old content', new_value: 'Updated content' },
      { field: 'version', old_value: 1, new_value: 2 },
    ],
    performed_by: 'agent-1',
    created_at: '2026-07-25T23:30:00Z',
  },
  {
    id: 'aud_000003',
    action: 'session.delete',
    entity_type: 'session',
    entity_id: 'ses_000005',
    changes: [{ field: 'status', old_value: 'done', new_value: undefined }],
    performed_by: 'user-001',
    created_at: '2026-07-25T22:00:00Z',
  },
  {
    id: 'aud_000004',
    action: 'agent.create',
    entity_type: 'agent',
    entity_id: 'agt_000002',
    changes: [{ field: 'name', old_value: undefined, new_value: 'Agent-2' }],
    performed_by: 'system',
    created_at: '2026-07-25T20:00:00Z',
  },
];

export const auditHandlers: HttpHandler[] = [
  // GET /api/v1/audit
  http.get('*/api/v1/audit', () => {
    return HttpResponse.json(auditLog);
  }),
];
