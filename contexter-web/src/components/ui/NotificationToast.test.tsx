import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { NotificationToast, type NotificationToastProps } from './NotificationToast';
import type { Notification } from '@/api/types';

/* ─── Fixtures ───────────────────────────────────────────────── */
const unreadNotification: Notification = {
  id: 'notif_000001',
  type: 'info',
  title: 'Session Complete',
  message: 'Your session ses_000001 has completed successfully.',
  read: false,
  created_at: '2026-07-24T12:00:00Z',
};

const readNotification: Notification = {
  id: 'notif_000002',
  type: 'warning',
  title: 'Low Memory',
  message: 'System memory usage has exceeded 85%. Consider freeing up resources.',
  read: true,
  created_at: '2026-07-25T08:30:00Z',
};

const errorNotification: Notification = {
  id: 'notif_000003',
  type: 'error',
  title: 'Export Failed',
  message: 'The requested export job exp_000001 has failed due to a processing error.',
  read: false,
  created_at: '2026-07-24T15:45:00Z',
};

const successNotification: Notification = {
  id: 'notif_000004',
  type: 'success',
  title: 'Sync Complete',
  message: 'All data has been synchronized successfully.',
  read: false,
  created_at: '2026-07-26T10:00:00Z',
};

/* ─── Shared render ──────────────────────────────────────────── */
function renderToast(notification: Notification, onMarkRead?: (id: string) => void) {
  const props: NotificationToastProps = { notification };
  if (onMarkRead) props.onMarkRead = onMarkRead;
  return render(<NotificationToast {...props} />);
}

/* ─── Tests ──────────────────────────────────────────────────── */
describe('NotificationToast', () => {
  it('renders notification title and message', () => {
    renderToast(unreadNotification);

    expect(screen.getByText('Session Complete')).toBeInTheDocument();
    expect(
      screen.getByText('Your session ses_000001 has completed successfully.'),
    ).toBeInTheDocument();
  });

  it('shows unread dot for unread notifications', () => {
    renderToast(unreadNotification);

    expect(screen.getByTestId('unread-dot')).toBeInTheDocument();
  });

  it('does not show unread dot for read notifications', () => {
    renderToast(readNotification);

    expect(screen.queryByTestId('unread-dot')).not.toBeInTheDocument();
  });

  it('renders relative timestamp', () => {
    renderToast(unreadNotification);

    // Should show a relative time string like "2 days ago" or "in about 2 days"
    const timestampEl = screen.getByText(/(ago|in about)/);
    expect(timestampEl).toBeInTheDocument();
  });

  it('shows mark-as-read button for unread notifications when onMarkRead provided', () => {
    const onMarkRead = vi.fn();
    renderToast(unreadNotification, onMarkRead);

    const btn = screen.getByTestId('mark-read-btn');
    expect(btn).toBeInTheDocument();
    btn.click();
    expect(onMarkRead).toHaveBeenCalledWith('notif_000001');
  });

  it('does not show mark-as-read button for read notifications', () => {
    renderToast(readNotification, vi.fn());

    expect(screen.queryByTestId('mark-read-btn')).not.toBeInTheDocument();
  });

  it('does not show mark-as-read button when onMarkRead is not provided', () => {
    renderToast(unreadNotification);

    expect(screen.queryByTestId('mark-read-btn')).not.toBeInTheDocument();
  });

  it('renders error-type notification with correct content', () => {
    renderToast(errorNotification);

    expect(screen.getByText('Export Failed')).toBeInTheDocument();
    expect(
      screen.getByText(
        'The requested export job exp_000001 has failed due to a processing error.',
      ),
    ).toBeInTheDocument();
  });

  it('renders success-type notification with correct content', () => {
    renderToast(successNotification);

    expect(screen.getByText('Sync Complete')).toBeInTheDocument();
    expect(
      screen.getByText('All data has been synchronized successfully.'),
    ).toBeInTheDocument();
  });

  it('uses data-testid based on notification id', () => {
    renderToast(unreadNotification);

    expect(screen.getByTestId('notification-toast-notif_000001')).toBeInTheDocument();
  });

  it('has role listitem for accessibility', () => {
    renderToast(unreadNotification);

    expect(screen.getByRole('listitem')).toBeInTheDocument();
  });
});
