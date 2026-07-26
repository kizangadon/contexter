import { useState, useMemo, useCallback } from 'react';
import { useNavigate, Link } from 'react-router';
import { Activity, Plus } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';
import { useSessions } from '@/api/hooks';
import type { Session } from '@/api/types';
import { PageHeader } from '@/components/layout/PageHeader';
import { FilterBar } from '@/components/ui/FilterBar';
import type { FilterDef } from '@/components/ui/FilterBar';
import { DataTable, type Column } from '@/components/ui/DataTable';
import { Badge, type BadgeVariant } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import { EmptyState } from '@/components/ui/EmptyState';

/* ── Status → Badge variant ────────────────────────────────── */
const statusVariant: Record<Session['status'], BadgeVariant> = {
  active: 'success',
  done: 'info',
  error: 'error',
  paused: 'pending',
};

/* ── Filter options ─────────────────────────────────────────── */
const STATUS_OPTIONS = [
  { value: '', label: 'All' },
  { value: 'active', label: 'Active' },
  { value: 'done', label: 'Done' },
  { value: 'error', label: 'Error' },
  { value: 'paused', label: 'Paused' },
] as const;

/* ── Page ───────────────────────────────────────────────────── */

export function SessionManagerPage() {
  const navigate = useNavigate();
  const [statusFilter, setStatusFilter] = useState('');
  const [sortConfig, setSortConfig] = useState<{
    key: string;
    direction: 'asc' | 'desc';
  } | null>(null);

  const { data, isLoading, isError, refetch } = useSessions(
    statusFilter ? { status: statusFilter } : undefined,
  );

  /* ── Filter definitions ─────────────────────────────────── */
  const filters: FilterDef[] = [
    {
      key: 'status',
      label: 'Status',
      options: [...STATUS_OPTIONS],
      value: statusFilter,
      onChange: setStatusFilter,
    },
  ];

  /* ── Client-side sorting ─────────────────────────────────── */
  const sortedSessions = useMemo(() => {
    if (!data) return [];
    if (!sortConfig) return data;

    return [...data].sort((a, b) => {
      let cmp = 0;
      switch (sortConfig.key) {
        case 'duration':
          cmp = a.duration_minutes - b.duration_minutes;
          break;
        case 'turns':
          cmp = a.turn_count - b.turn_count;
          break;
        case 'last_active':
          cmp = a.last_active.localeCompare(b.last_active);
          break;
        default:
          return 0;
      }
      return sortConfig.direction === 'asc' ? cmp : -cmp;
    });
  }, [data, sortConfig]);

  const handleSort = useCallback(
    (key: string, direction: 'asc' | 'desc') => {
      setSortConfig({ key, direction });
    },
    [],
  );

  /* ── Row click → navigate ────────────────────────────────── */
  const handleRowClick = useCallback(
    (session: Session) => {
      navigate(`/sessions/${session.id}`);
    },
    [navigate],
  );

  /* ── Column definitions ──────────────────────────────────── */
  const columns: Column<Session>[] = [
    {
      key: 'id',
      header: 'ID',
      render: (s) => (
        <span className="font-mono text-xs text-text-secondary" title={s.id}>
          {s.id.length > 12 ? `${s.id.slice(0, 12)}…` : s.id}
        </span>
      ),
    },
    {
      key: 'agent',
      header: 'Agent',
      render: (s) => <span className="text-text-primary">{s.agent}</span>,
    },
    {
      key: 'status',
      header: 'Status',
      render: (s) => (
        <Badge variant={statusVariant[s.status]} size="sm" dot>
          {s.status}
        </Badge>
      ),
      width: '100px',
    },
    {
      key: 'duration',
      header: 'Duration',
      sortable: true,
      render: (s) => (
        <span className="text-text-secondary">{s.duration_minutes}m</span>
      ),
      width: '100px',
    },
    {
      key: 'turns',
      header: 'Turns',
      sortable: true,
      render: (s) => (
        <span className="text-text-secondary">{s.turn_count}</span>
      ),
      width: '80px',
    },
    {
      key: 'last_active',
      header: 'Last Active',
      sortable: true,
      render: (s) => (
        <span className="text-text-secondary" title={s.last_active}>
          {formatDistanceToNow(new Date(s.last_active), {
            addSuffix: true,
          })}
        </span>
      ),
    },
  ];

  /* ── Render ──────────────────────────────────────────────── */

  return (
    <div>
      {/* Page Header with "New Session" button */}
      <PageHeader title="Sessions">
        <Link to="/sessions">
          <Button variant="primary">
            <Plus className="h-4 w-4" />
            New Session
          </Button>
        </Link>
      </PageHeader>

      {/* Filter bar */}
      <div className="mb-lg">
        <FilterBar filters={filters} />
      </div>

      {/* Error state */}
      {isError ? (
        <EmptyState
          icon={Activity}
          title="Failed to load sessions"
          message="Something went wrong while fetching sessions. Please try again."
          action={
            <Button variant="primary" onClick={() => refetch()}>
              Retry
            </Button>
          }
        />
      ) : (
        <DataTable<Session>
          columns={columns}
          data={sortedSessions}
          isLoading={isLoading}
          sortable
          onSort={handleSort}
          onRowClick={handleRowClick}
          pageSize={10}
          emptyState={{
            icon: Activity,
            title: 'No sessions yet',
            message:
              'Create your first session to get started with Contexter.',
            action: (
              <Link to="/sessions">
                <Button variant="primary">
                  <Plus className="h-4 w-4" />
                  Create Session
                </Button>
              </Link>
            ),
          }}
        />
      )}
    </div>
  );
}
