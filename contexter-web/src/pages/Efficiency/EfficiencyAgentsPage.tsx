import { useState, useCallback } from 'react';
import { RefreshCw, TrendingUp } from 'lucide-react';
import { useEfficiencyAgents } from '@/api/hooks';
import type { AgentPerformance } from '@/api/types';
import { DataTable, type Column } from '@/components/ui/DataTable';
import { PageHeader } from '@/components/layout/PageHeader';
import { StatCard } from '@/components/ui/StatCard';
import { LoadingSkeleton } from '@/components/ui/LoadingSkeleton';
import { TimeframeFilter } from '@/components/ui/TimeframeFilter';
import { Button } from '@/components/ui/Button';
import { formatNumber } from '@/utils/formatters';

const columns: Column<AgentPerformance>[] = [
  {
    key: 'agent_name',
    header: 'Agent',
    render: (a) => (
      <span className="font-medium text-text-primary">{a.agent_name}</span>
    ),
  },
  {
    key: 'efficiency_score',
    header: 'Efficiency',
    render: (a) => (
      <span className="text-text-secondary">{a.efficiency_score}%</span>
    ),
  },
  {
    key: 'sessions_count',
    header: 'Sessions',
    render: (a) => (
      <span className="text-text-secondary">{formatNumber(a.sessions_count)}</span>
    ),
  },
  {
    key: 'avg_latency_ms',
    header: 'Latency',
    render: (a) => (
      <span className="text-text-secondary">{a.avg_latency_ms}ms</span>
    ),
  },
  {
    key: 'trend',
    header: 'Trend',
    render: (a) => {
      const color = a.trend > 0 ? 'text-success' : a.trend < 0 ? 'text-error' : 'text-text-tertiary';
      const sign = a.trend > 0 ? '+' : '';
      return (
        <span className={`font-medium ${color}`}>
          {sign}{a.trend}%
        </span>
      );
    },
  },
];

export function EfficiencyAgentsPage() {
  const [timeframe, setTimeframe] = useState('30d');
  const { data, isLoading, error, refetch } = useEfficiencyAgents(timeframe);

  const handleRetry = useCallback(() => {
    refetch();
  }, [refetch]);

  /* ── Loading state ──────────────────────────────────────── */
  if (isLoading && !error) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader title="Agent Performance">
          <TimeframeFilter value={timeframe} onChange={setTimeframe} />
        </PageHeader>
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {Array.from({ length: 3 }, (_, i) => (
            <LoadingSkeleton key={i} variant="card" />
          ))}
        </div>
        <LoadingSkeleton variant="card" />
      </div>
    );
  }

  /* ── Error state ────────────────────────────────────────── */
  if (error) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader title="Agent Performance">
          <TimeframeFilter value={timeframe} onChange={setTimeframe} />
        </PageHeader>
        <div className="flex flex-col items-center justify-center gap-4 rounded-lg border border-border bg-surface p-8 text-center">
          <div className="rounded-full bg-error/10 p-3">
            <TrendingUp className="h-6 w-6 text-error" aria-hidden="true" />
          </div>
          <h3 className="text-lg font-semibold text-text-primary">
            Failed to load agent data
          </h3>
          <p className="max-w-sm text-sm text-text-secondary">
            Unable to retrieve agent performance metrics. Please try again.
          </p>
          <Button variant="primary" onClick={handleRetry}>
            <RefreshCw className="h-4 w-4" />
            Retry
          </Button>
        </div>
      </div>
    );
  }

  const agents: AgentPerformance[] = data ?? [];

  /* ── Derived stats ──────────────────────────────────────── */
  const avgEfficiency =
    agents.length > 0
      ? Math.round(
          agents.reduce((sum, a) => sum + a.efficiency_score, 0) /
            agents.length,
        )
      : 0;

  return (
    <div className="flex flex-col gap-lg">
      <PageHeader title="Agent Performance">
        <TimeframeFilter value={timeframe} onChange={setTimeframe} />
      </PageHeader>

      {/* Summary stat cards */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        <StatCard
          value={formatNumber(agents.length)}
          label="Total Agents"
        />
        <StatCard
          value={`${avgEfficiency}%`}
          label="Avg Efficiency"
        />
        <StatCard
          value={formatNumber(agents.reduce((sum, a) => sum + a.sessions_count, 0))}
          label="Total Sessions"
        />
      </div>

      {/* Agents table */}
      {agents.length > 0 ? (
        <DataTable<AgentPerformance>
          columns={columns}
          data={agents}
          pageSize={20}
        />
      ) : (
        <p className="text-sm text-text-tertiary">
          No agent performance data available for the selected timeframe.
        </p>
      )}
    </div>
  );
}
