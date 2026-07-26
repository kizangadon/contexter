import { useState, useCallback } from 'react';
import { Download, RefreshCw, Plus } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';
import { useExports, useSubmitExport } from '@/api/hooks';
import type { ExportJob } from '@/api/types';
import { PageHeader } from '@/components/layout/PageHeader';
import { DataTable, type Column } from '@/components/ui/DataTable';
import { Button } from '@/components/ui/Button';
import { Badge, type BadgeVariant } from '@/components/ui/Badge';
import { Modal } from '@/components/ui/Modal';

/* ─── Status → badge ────────────────────────────────────────── */
const statusVariant: Record<ExportJob['status'], BadgeVariant> = {
  pending: 'pending',
  processing: 'info',
  completed: 'success',
  failed: 'error',
};

/* ─── Columns ───────────────────────────────────────────────── */
const columns: Column<ExportJob>[] = [
  {
    key: 'type',
    header: 'Type',
    render: (e) => <span className="text-text-primary">{e.type}</span>,
  },
  {
    key: 'format',
    header: 'Format',
    render: (e) => (
      <span className="font-mono text-xs uppercase text-text-secondary">{e.format}</span>
    ),
  },
  {
    key: 'status',
    header: 'Status',
    render: (e) => (
      <Badge variant={statusVariant[e.status]} size="sm" dot>
        {e.status}
      </Badge>
    ),
  },
  {
    key: 'created_at',
    header: 'Created',
    render: (e) => (
      <span className="text-sm text-text-secondary">
        {formatDistanceToNow(new Date(e.created_at), { addSuffix: true })}
      </span>
    ),
  },
  {
    key: 'download',
    header: '',
    width: '80px',
    render: (e) =>
      e.status === 'completed' && e.download_url ? (
        <a
          href={e.download_url}
          className="inline-flex items-center gap-1 text-sm font-medium text-accent hover:text-accent-hover"
        >
          <Download className="h-4 w-4" />
          Download
        </a>
      ) : e.status === 'failed' ? (
        <span className="text-xs text-text-tertiary" title={e.error}>
          Failed
        </span>
      ) : null,
  },
];

/* ─── Component ──────────────────────────────────────────────── */

export function ExportsPage() {
  const [showModal, setShowModal] = useState(false);

  const exports = useExports();
  const submitExport = useSubmitExport();

  const [exportType, setExportType] = useState<ExportJob['type']>('sessions');
  const [exportFormat, setExportFormat] = useState<ExportJob['format']>('json');

  const isLoading = exports.isLoading;
  const isError = exports.isError;
  const data = exports.data ?? [];

  const handleNewExport = useCallback(() => {
    submitExport.mutate(
      { type: exportType, format: exportFormat },
      { onSuccess: () => setShowModal(false) },
    );
  }, [submitExport, exportType, exportFormat]);

  const handleRetry = useCallback(() => {
    exports.refetch();
  }, [exports]);

  return (
    <div className="flex flex-col gap-lg">
      <PageHeader title="Exports">
        <Button variant="primary" onClick={() => setShowModal(true)}>
          <Plus className="h-4 w-4" />
          New Export
        </Button>
      </PageHeader>

      {/* Error State */}
      {isError && (
        <div className="flex flex-col items-center justify-center gap-4 rounded-lg border border-border bg-surface p-8 text-center">
          <div className="rounded-full bg-error/10 p-3">
            <RefreshCw className="h-6 w-6 text-error" aria-hidden="true" />
          </div>
          <h3 className="text-lg font-semibold text-text-primary">Failed to load exports</h3>
          <p className="max-w-sm text-sm text-text-secondary">
            Something went wrong while fetching export jobs.
          </p>
          <Button variant="primary" onClick={handleRetry}>
            <RefreshCw className="h-4 w-4" />
            Retry
          </Button>
        </div>
      )}

      {/* Table */}
      {!isError && (
        <DataTable<ExportJob>
          columns={columns}
          data={data}
          isLoading={isLoading}
          emptyState={{
            icon: Download,
            title: 'No exports yet',
            message: 'Export your sessions, memories, or analytics data.',
          }}
        />
      )}

      {/* New Export Modal */}
      <Modal isOpen={showModal} onClose={() => setShowModal(false)} title="New Export">
          <div className="flex flex-col gap-4">
            <div>
              <label htmlFor="export-type" className="mb-1 block text-sm font-medium text-text-primary">
                Data Type
              </label>
              <select
                id="export-type"
                value={exportType}
                onChange={(e) => setExportType(e.target.value as ExportJob['type'])}
                className="w-full rounded-md border border-border bg-bg-primary px-3 py-2 text-sm text-text-primary focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent"
              >
                <option value="sessions">Sessions</option>
                <option value="memories">Memories</option>
                <option value="analytics">Analytics</option>
              </select>
            </div>
            <div>
              <label htmlFor="export-format" className="mb-1 block text-sm font-medium text-text-primary">
                Format
              </label>
              <select
                id="export-format"
                value={exportFormat}
                onChange={(e) => setExportFormat(e.target.value as ExportJob['format'])}
                className="w-full rounded-md border border-border bg-bg-primary px-3 py-2 text-sm text-text-primary focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent"
              >
                <option value="json">JSON</option>
                <option value="csv">CSV</option>
              </select>
            </div>
            <div className="flex justify-end gap-2">
              <Button variant="secondary" onClick={() => setShowModal(false)}>
                Cancel
              </Button>
              <Button
                variant="primary"
                onClick={handleNewExport}
                loading={submitExport.isPending}
              >
                Start Export
              </Button>
            </div>
          </div>
        </Modal>
    </div>
  );
}
