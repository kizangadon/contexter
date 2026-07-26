import { useState } from 'react';
import { Link } from 'react-router';
import { Activity, BarChart3, RefreshCw, Rocket, Search, TrendingUp } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';
import { useSessions, useMemories, useEfficiencyOverview } from '@/api/hooks';
import { StatCard, type Trend } from '@/components/ui/StatCard';
import { DataTable, type Column } from '@/components/ui/DataTable';
import { PageHeader } from '@/components/layout/PageHeader';
import { Badge, type BadgeVariant } from '@/components/ui/Badge';
import { EmptyState } from '@/components/ui/EmptyState';
import { Button } from '@/components/ui/Button';
import { TimeframeFilter } from '@/components/ui/TimeframeFilter';
import type { Session } from '@/api/types';

/* ─── Status → Badge variant mapping ──────────────────────── */
const statusVariant: Record<Session['status'], BadgeVariant> = {
  active: 'success',
  done: 'info',
  error: 'error',
  paused: 'pending',
};

/* ─── Trend helper ─────────────────────────────────────────── */
function trendFromValue(value: number): Trend['direction'] {
  if (value > 0) return 'up';
  if (value < 0) return 'down';
  return 'neutral';
}

/* ─── Table columns ────────────────────────────────────────── */
const sessionColumns: Column<Session>[] = [
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
  },
  {
    key: 'duration',
    header: 'Duration',
    render: (s) => <span className="text-text-secondary">{s.duration_minutes}m</span>,
  },
  {
    key: 'turns',
    header: 'Turns',
    render: (s) => <span className="text-text-secondary">{s.turn_count}</span>,
  },
  {
    key: 'last_active',
    header: 'Last Active',
    render: (s) => (
      <span className="text-text-secondary" title={s.last_active}>
        {formatDistanceToNow(new Date(s.last_active), { addSuffix: true })}
      </span>
    ),
  },
];

/* ─── Quick action config ──────────────────────────────────── */
interface QuickAction {
  icon: typeof Rocket;
  label: string;
  description: string;
  to: string;
}

const quickActions: QuickAction[] = [
  { icon: Rocket, label: 'Launch Session', description: 'Start a new agent session', to: '/sessions' },
  { icon: Search, label: 'Explore Memories', description: 'Browse stored knowledge', to: '/memories' },
  { icon: BarChart3, label: 'View Analytics', description: 'Review performance metrics', to: '/analytics' },
];

/* ─── Component ────────────────────────────────────────────── */

export function DashboardPage() {
  const [timeframe, setTimeframe] = useState('30d');
  const sessions = useSessions();
  const memories = useMemories();
  const efficiency = useEfficiencyOverview(timeframe);

  /* ── Derived data ─────────────────────────────────────── */
  const isLoading = sessions.isLoading || memories.isLoading || efficiency.isLoading;
  const isError = sessions.isError || memories.isError || efficiency.isError;
  const totalSessions = sessions.data?.length ?? 0;
  const activeSessions = sessions.data?.filter((s) => s.status === 'active').length ?? 0;
  const totalMemories = memories.data?.length ?? 0;
  const avgEfficiency = efficiency.data?.avg_efficiency ?? 0;
  const efficiencyTrend = efficiency.data?.trend ?? 0;
  const recentSessions = sessions.data?.slice(0, 5) ?? [];
  const noSessions = !isLoading && !isError && totalSessions === 0;

  /* ── Trends for stat cards ────────────────────────────── */
  const trends: Record<string, Trend> = {
    totalSessions: totalSessions > 0 ? { direction: 'up', percentage: totalSessions } : { direction: 'neutral', percentage: 0 },
    activeSessions: { direction: 'neutral', percentage: 0 },
    totalMemories: totalMemories > 0 ? { direction: 'up', percentage: totalMemories } : { direction: 'neutral', percentage: 0 },
    avgEfficiency: { direction: trendFromValue(efficiencyTrend), percentage: Math.abs(efficiencyTrend) },
  };

  /* ── Retry handler ────────────────────────────────────── */
  const handleRetry = () => {
    sessions.refetch();
    memories.refetch();
    efficiency.refetch();
  };

  /* ── Render ───────────────────────────────────────────── */

  return (
    <div className="flex flex-col gap-lg">
      {/* Page Header */}
      <PageHeader title="Dashboard">
        <TimeframeFilter value={timeframe} onChange={setTimeframe} />
      </PageHeader>

      {/* Error State */}
      {isError && (
        <div className="flex flex-col items-center justify-center gap-4 rounded-lg border border-border bg-surface p-8 text-center">
          <div className="rounded-full bg-error/10 p-3">
            <TrendingUp className="h-6 w-6 text-error" aria-hidden="true" />
          </div>
          <h3 className="text-lg font-semibold text-text-primary">Failed to load dashboard</h3>
          <p className="max-w-sm text-sm text-text-secondary">
            Something went wrong while fetching your data. Please try again.
          </p>
          <Button variant="primary" onClick={handleRetry}>
            <RefreshCw className="h-4 w-4" />
            Retry
          </Button>
        </div>
      )}

      {/* Stat Cards */}
      {!isError && (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <StatCard
            value={isLoading ? '—' : totalSessions}
            label="Total Sessions"
            trend={trends.totalSessions}
            loading={isLoading}
          />
          <StatCard
            value={isLoading ? '—' : activeSessions}
            label="Active Sessions"
            trend={trends.activeSessions}
            loading={isLoading}
          />
          <StatCard
            value={isLoading ? '—' : totalMemories}
            label="Total Memories"
            trend={trends.totalMemories}
            loading={isLoading}
          />
          <StatCard
            value={isLoading ? '—' : avgEfficiency}
            label="Avg Efficiency"
            trend={trends.avgEfficiency}
            loading={isLoading}
          />
        </div>
      )}

      {/* Recent Sessions */}
      {!isError && (
        <section className="flex flex-col gap-4">
          <div className="flex items-center justify-between">
            <h2 className="text-lg font-semibold text-text-primary">Recent Sessions</h2>
          </div>

          {noSessions ? (
            <div className="rounded-lg border border-border">
              <EmptyState
                icon={Activity}
                title="No sessions yet"
                message="Create your first session to get started"
                action={
                  <Link to="/sessions">
                    <Button variant="primary">Create your first session</Button>
                  </Link>
                }
              />
            </div>
          ) : (
            <>
              <DataTable<Session>
                columns={sessionColumns}
                data={recentSessions}
                isLoading={isLoading}
                pageSize={5}
              />
              <div className="flex justify-end">
                <Link
                  to="/sessions"
                  className="text-sm font-medium text-accent transition-colors hover:text-accent-hover"
                >
                  View All &rarr;
                </Link>
              </div>
            </>
          )}
        </section>
      )}

      {/* Quick Actions */}
      <section className="flex flex-col gap-4">
        <h2 className="text-lg font-semibold text-text-primary">Quick Actions</h2>
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
          {quickActions.map((action) => {
            const Icon = action.icon;
            return (
              <Link
                key={action.label}
                to={action.to}
                className="group flex flex-col gap-3 rounded-lg border border-border bg-surface p-5 transition-colors hover:border-accent/30 hover:bg-surface-hover"
              >
                <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-accent/10 text-accent transition-colors group-hover:bg-accent/15">
                  <Icon className="h-5 w-5" aria-hidden="true" />
                </div>
                <div>
                  <h3 className="text-sm font-semibold text-text-primary">{action.label}</h3>
                  <p className="mt-0.5 text-xs text-text-secondary">{action.description}</p>
                </div>
              </Link>
            );
          })}
        </div>
      </section>
    </div>
  );
}
