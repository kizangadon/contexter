import { http, HttpResponse } from 'msw';
import type { HttpHandler } from 'msw';
import type { Notification } from '@/api/types';

const notifications: Notification[] = [
  { id: 'not_000001', type: 'info', title: 'Session Complete', message: 'Session ses_000001 completed successfully.', read: false, created_at: '2026-07-26T00:30:00Z' },
  { id: 'not_000002', type: 'warning', title: 'High Memory Usage', message: 'Memory usage exceeded 80% threshold.', read: false, created_at: '2026-07-26T01:00:00Z' },
  { id: 'not_000003', type: 'success', title: 'Export Ready', message: 'Your analytics export is ready for download.', read: true, created_at: '2026-07-25T23:00:00Z' },
  { id: 'not_000004', type: 'error', title: 'Agent Error', message: 'Agent-3 encountered an unexpected error.', read: false, created_at: '2026-07-25T22:45:00Z' },
  { id: 'not_000005', type: 'info', title: 'New Skill Available', message: 'A new skill "Test Sage" has been added.', read: true, created_at: '2026-07-25T20:00:00Z' },
];

export const notificationsHandlers: HttpHandler[] = [
  // GET /api/v1/notifications
  http.get('*/api/v1/notifications', () => {
    return HttpResponse.json(notifications);
  }),

  // GET /api/v1/notifications/unread-count
  http.get('*/api/v1/notifications/unread-count', () => {
    const count = notifications.filter((n) => !n.read).length;
    return HttpResponse.json({ count });
  }),

  // PATCH /api/v1/notifications/:id/read — mark as read
  http.patch('*/api/v1/notifications/:id/read', ({ params }) => {
    const idx = notifications.findIndex((n) => n.id === params.id);
    if (idx === -1) {
      return HttpResponse.json({ detail: 'Notification not found' }, { status: 404 });
    }
    notifications[idx] = { ...notifications[idx]!, read: true };
    return HttpResponse.json(notifications[idx]);
  }),

  // POST /api/v1/notifications/read-all — mark all as read
  http.post('*/api/v1/notifications/read-all', () => {
    for (let i = 0; i < notifications.length; i++) {
      notifications[i] = { ...notifications[i]!, read: true };
    }
    return HttpResponse.json({ success: true });
  }),
];
