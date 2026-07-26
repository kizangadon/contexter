import { useCallback, useState } from 'react';
import { RefreshCw, TrendingUp } from 'lucide-react';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from 'recharts';
import {
  useAnalyticsOverview,
  useAnalyticsHealth,
  useAnalyticsPerformance,
  useAnalyticsResources,
  useAnalyticsCosts,
} from '@/api/hooks';
import type { HealthStatus } from '@/api/hooks';
import type {
  AnalyticsOverview,
  PerformanceTrend,
  ResourceUsage,
  CostBreakdown,
} from '@/api/types';
import { PageHeader } from '@/components/layout/PageHeader';
import { StatCard } from '@/components/ui/StatCard';
import { Badge } from '@/components/ui/Badge';
import { LoadingSkeleton } from '@/components/ui/LoadingSkeleton';
import { TimeframeFilter } from '@/components/ui/TimeframeFilter';
import { Button } from '@/components/ui/Button';
import { formatNumber, formatCurrency, formatPercent } from '@/utils/formatters';

/* ─── Status badge variant mapping ─────────────────────────── */
type ServiceStatusVariant = 'success' | 'warning' | 'error' | 'info' | 'offline';

function statusToVariant(
  status: string,
): ServiceStatusVariant {
  switch (status) {
    case 'healthy':
      return 'success';
    case 'degraded':
      return 'warning';
    case 'down':
      return 'error';
    default:
      return 'info';
  }
}

/* ─── Component ────────────────────────────────────────────── */

export function AnalyticsDashboardPage() {
  const [timeframe, setTimeframe] = useState('30d');

  const overview = useAnalyticsOverview(timeframe);
  const health = useAnalyticsHealth();
  const performance = useAnalyticsPerformance(timeframe);
  const resources = useAnalyticsResources();
  const costs = useAnalyticsCosts(timeframe);

  /* ── Derived state ─────────────────────────────────────── */
  const isLoading =
    overview.isLoading ||
    performance.isLoading ||
    resources.isLoading ||
    costs.isLoading;

  const isError =
    overview.isError ||
    performance.isError ||
    resources.isError ||
    costs.isError;

  const overviewData: AnalyticsOverview | undefined = overview.data;
  const healthData: HealthStatus | undefined = health.data;
  const performanceData: PerformanceTrend[] = performance.data ?? [];
  const resourcesData: ResourceUsage | undefined = resources.data;
  const costsData: CostBreakdown | undefined = costs.data;

  /* ── Retry handler ──────────────────────────────────────── */
  const handleRetry = useCallback(() => {
    overview.refetch();
    health.refetch();
    performance.refetch();
    resources.refetch();
    costs.refetch();
  }, [overview, health, performance, resources, costs]);

  /* ── Render loading state ───────────────────────────────── */
  if (isLoading && !isError) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader title="Analytics">
          <TimeframeFilter value={timeframe} onChange={setTimeframe} />
        </PageHeader>

        {/* Stat card skeletons */}
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {Array.from({ length: 6 }, (_, i) => (
            <LoadingSkeleton key={i} variant="card" />
          ))}
        </div>

        {/* Content skeletons */}
        <LoadingSkeleton variant="card" count={3} />
      </div>
    );
  }

  /* ── Render error state ─────────────────────────────────── */
  if (isError) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader title="Analytics">
          <TimeframeFilter value={timeframe} onChange={setTimeframe} />
        </PageHeader>

        <div className="flex flex-col items-center justify-center gap-4 rounded-lg border border-border bg-surface p-8 text-center">
          <div className="rounded-full bg-error/10 p-3">
            <TrendingUp className="h-6 w-6 text-error" aria-hidden="true" />
          </div>
          <h3 className="text-lg font-semibold text-text-primary">
            Failed to load analytics
          </h3>
          <p className="max-w-sm text-sm text-text-secondary">
            Something went wrong while fetching analytics data. Please try
            again.
          </p>
          <Button variant="primary" onClick={handleRetry}>
            <RefreshCw className="h-4 w-4" />
            Retry
          </Button>
        </div>
      </div>
    );
  }

  /* ── Render data state ──────────────────────────────────── */

  // Health services from either health.services or the services endpoint
  const services: { name: string; status: string }[] = [];
  if (healthData?.services) {
    for (const [name, status] of Object.entries(healthData.services)) {
      services.push({ name, status: status as string });
    }
  }

  return (
    <div className="flex flex-col gap-lg">
      {/* Header + Timeframe Filter */}
      <PageHeader title="Analytics">
        <TimeframeFilter value={timeframe} onChange={setTimeframe} />
      </PageHeader>

      {/* 6 Stat Cards in 3×2 grid */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        <StatCard
          value={overviewData?.system_health ?? '—'}
          label="System Health"
        />
        <StatCard
          value={overviewData ? formatPercent(overviewData.uptime_percent) : '—'}
          label="Uptime"
        />
        <StatCard
          value={overviewData ? formatPercent(overviewData.error_rate) : '—'}
          label="Error Rate"
        />
        <StatCard
          value={overviewData ? formatNumber(overviewData.active_sessions) : '—'}
          label="Active Sessions"
        />
        <StatCard
          value={
            overviewData
              ? formatPercent(overviewData.memory_usage_percent)
              : '—'
          }
          label="Memory Usage"
        />
        <StatCard
          value={overviewData ? formatCurrency(overviewData.cost_total) : '—'}
          label="Total Cost"
        />
      </div>

      {/* System Status / Health Section */}
      <section className="flex flex-col gap-4">
        <h2 className="text-lg font-semibold text-text-primary">
          System Status
        </h2>
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5">
          {services.map((svc) => (
            <div
              key={svc.name}
              className="flex items-center justify-between rounded-lg border border-border bg-surface p-3"
            >
              <span className="text-sm font-medium text-text-primary">
                {svc.name}
              </span>
              <Badge variant={statusToVariant(svc.status)} size="sm" dot>
                {svc.status}
              </Badge>
            </div>
          ))}
        </div>
      </section>

      {/* Performance Trend Section */}
      <section className="flex flex-col gap-4">
        <h2 className="text-lg font-semibold text-text-primary">
          Performance Trend
        </h2>
        <div className="rounded-lg border border-border bg-surface p-4">
          <div className="h-64 w-full">
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={performanceData}>
                <CartesianGrid
                  strokeDasharray="3 3"
                  stroke="var(--color-border)"
                />
                <XAxis
                  dataKey="date"
                  stroke="var(--color-text-tertiary)"
                  fontSize={12}
                />
                <YAxis
                  stroke="var(--color-text-tertiary)"
                  fontSize={12}
                  unit="ms"
                />
                <Tooltip
                  contentStyle={{
                    background: 'var(--color-surface)',
                    border: '1px solid var(--color-border)',
                    borderRadius: '8px',
                    fontSize: '13px',
                  }}
                />
                <Line
                  type="monotone"
                  dataKey="response_time_ms"
                  name="Response Time"
                  stroke="var(--color-accent)"
                  strokeWidth={2}
                  dot={{ r: 4, fill: 'var(--color-accent)' }}
                  activeDot={{ r: 6 }}
                />
              </LineChart>
            </ResponsiveContainer>
          </div>
        </div>
      </section>

      {/* Resource Usage Section */}
      <section className="flex flex-col gap-4">
        <h2 className="text-lg font-semibold text-text-primary">
          Resource Usage
        </h2>
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <ResourceCard
            label="CPU"
            value={resourcesData ? formatPercent(resourcesData.cpu_percent) : '—'}
            percent={resourcesData?.cpu_percent}
          />
          <ResourceCard
            label="Memory"
            value={
              resourcesData ? formatPercent(resourcesData.memory_percent) : '—'
            }
            percent={resourcesData?.memory_percent}
          />
          <ResourceCard
            label="Disk"
            value={
              resourcesData ? formatPercent(resourcesData.disk_percent) : '—'
            }
            percent={resourcesData?.disk_percent}
          />
          <ResourceCard
            label="Active Connections"
            value={
              resourcesData
                ? formatNumber(resourcesData.active_connections)
                : '—'
            }
          />
        </div>
      </section>

      {/* Cost Overview Section */}
      <section className="flex flex-col gap-4">
        <h2 className="text-lg font-semibold text-text-primary">
          Cost Overview
        </h2>

        {/* By-model breakdown */}
        {costsData && costsData.by_model.length > 0 ? (
          <div className="rounded-lg border border-border bg-surface">
            <table className="w-full border-collapse text-sm">
              <thead>
                <tr className="border-b border-border">
                  <th className="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-text-secondary">
                    Model
                  </th>
                  <th className="px-4 py-3 text-right text-xs font-semibold uppercase tracking-wider text-text-secondary">
                    Cost
                  </th>
                  <th className="px-4 py-3 text-right text-xs font-semibold uppercase tracking-wider text-text-secondary">
                    Tokens
                  </th>
                  <th className="px-4 py-3 text-right text-xs font-semibold uppercase tracking-wider text-text-secondary">
                    %
                  </th>
                </tr>
              </thead>
              <tbody>
                {costsData.by_model.map((m) => (
                  <tr
                    key={m.model}
                    className="border-b border-border last:border-b-0 hover:bg-bg-hover"
                  >
                    <td className="px-4 py-3 font-medium text-text-primary">
                      {m.model}
                    </td>
                    <td className="px-4 py-3 text-right text-text-secondary">
                      {formatCurrency(m.cost)}
                    </td>
                    <td className="px-4 py-3 text-right text-text-secondary">
                      {formatNumber(m.tokens)}
                    </td>
                    <td className="px-4 py-3 text-right text-text-secondary">
                      {m.percentage.toFixed(1)}%
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <p className="text-sm text-text-tertiary">
            No cost data available for the selected timeframe.
          </p>
        )}
      </section>
    </div>
  );
}

/* ─── Resource Card Sub-component ──────────────────────────── */

function ResourceCard({
  label,
  value,
  percent,
}: {
  label: string;
  value: string | number;
  percent?: number;
}) {
  const barColor =
    percent !== undefined
      ? percent > 80
        ? 'bg-error'
        : percent > 60
          ? 'bg-warning'
          : 'bg-accent'
      : 'bg-bg-tertiary';

  return (
    <div className="flex flex-col gap-2 rounded-lg border border-border bg-surface p-4">
      <span className="text-sm text-text-secondary">{label}</span>
      <span className="text-2xl font-bold text-text-primary">{value}</span>
      {percent !== undefined && (
        <div className="mt-1 h-2 w-full overflow-hidden rounded-full bg-bg-tertiary">
          <div
            className={`h-full rounded-full transition-all duration-300 ${barColor}`}
            style={{ width: `${Math.min(percent, 100)}%` }}
          />
        </div>
      )}
    </div>
  );
}
