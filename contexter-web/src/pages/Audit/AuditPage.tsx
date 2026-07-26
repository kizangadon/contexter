import { useCallback } from 'react';
import { ClipboardList, RefreshCw } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';
import { useAudit } from '@/api/hooks';
import type { AuditEntry } from '@/api/types';
import { PageHeader } from '@/components/layout/PageHeader';
import { DataTable, type Column } from '@/components/ui/DataTable';
import { Button } from '@/components/ui/Button';

/* ─── Columns ───────────────────────────────────────────────── */
const columns: Column<AuditEntry>[] = [
  {
    key: 'created_at',
    header: 'Timestamp',
    render: (e) => (
      <span className="text-sm text-text-secondary" title={e.created_at}>
        {formatDistanceToNow(new Date(e.created_at), { addSuffix: true })}
      </span>
    ),
  },
  {
    key: 'action',
    header: 'Action',
    render: (e) => (
      <span className="font-mono text-xs font-medium text-text-primary">{e.action}</span>
    ),
  },
  {
    key: 'entity_type',
    header: 'Entity',
    render: (e) => (
      <span className="text-sm text-text-secondary">{e.entity_type}</span>
    ),
  },
  {
    key: 'performed_by',
    header: 'User',
    render: (e) => (
      <span className="text-sm text-text-secondary">{e.performed_by}</span>
    ),
  },
  {
    key: 'changes',
    header: 'Details',
    render: (e) => (
      <span className="text-xs text-text-tertiary">
        {e.changes.map((c) => c.field).join(', ') || '—'}
      </span>
    ),
  },
];

/* ─── Component ──────────────────────────────────────────────── */

export function AuditPage() {
  const audit = useAudit();

  const isLoading = audit.isLoading;
  const isError = audit.isError;
  const data = audit.data ?? [];

  const handleRetry = useCallback(() => {
    audit.refetch();
  }, [audit]);

  return (
    <div className="flex flex-col gap-lg">
      <PageHeader title="Audit Log" />

      {/* Error State */}
      {isError && (
        <div className="flex flex-col items-center justify-center gap-4 rounded-lg border border-border bg-surface p-8 text-center">
          <div className="rounded-full bg-error/10 p-3">
            <ClipboardList className="h-6 w-6 text-error" aria-hidden="true" />
          </div>
          <h3 className="text-lg font-semibold text-text-primary">Failed to load audit log</h3>
          <p className="max-w-sm text-sm text-text-secondary">
            Something went wrong while fetching audit entries.
          </p>
          <Button variant="primary" onClick={handleRetry}>
            <RefreshCw className="h-4 w-4" />
            Retry
          </Button>
        </div>
      )}

      {/* Table */}
      {!isError && (
        <DataTable<AuditEntry>
          columns={columns}
          data={data}
          isLoading={isLoading}
          emptyState={{
            icon: ClipboardList,
            title: 'No audit entries',
            message: 'Audit log entries will appear here as actions are performed in the system.',
          }}
        />
      )}
    </div>
  );
}
