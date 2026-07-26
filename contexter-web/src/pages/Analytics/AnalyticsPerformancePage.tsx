import { useState, useCallback } from 'react';
import { RefreshCw, TrendingUp } from 'lucide-react';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from 'recharts';
import { useAnalyticsPerformance } from '@/api/hooks';
import type { PerformanceTrend } from '@/api/types';
import { PageHeader } from '@/components/layout/PageHeader';
import { StatCard } from '@/components/ui/StatCard';
import { LoadingSkeleton } from '@/components/ui/LoadingSkeleton';
import { TimeframeFilter } from '@/components/ui/TimeframeFilter';
import { Button } from '@/components/ui/Button';
import { formatNumber } from '@/utils/formatters';

export function AnalyticsPerformancePage() {
  const [timeframe, setTimeframe] = useState('30d');
  const { data, isLoading, error, refetch } = useAnalyticsPerformance(timeframe);

  const handleRetry = useCallback(() => {
    refetch();
  }, [refetch]);

  /* ── Loading state ──────────────────────────────────────── */
  if (isLoading && !error) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader title="Performance Trends">
          <TimeframeFilter value={timeframe} onChange={setTimeframe} />
        </PageHeader>
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {Array.from({ length: 3 }, (_, i) => (
            <LoadingSkeleton key={i} variant="card" />
          ))}
        </div>
        <LoadingSkeleton variant="card" count={2} />
      </div>
    );
  }

  /* ── Error state ────────────────────────────────────────── */
  if (error) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader title="Performance Trends">
          <TimeframeFilter value={timeframe} onChange={setTimeframe} />
        </PageHeader>
        <div className="flex flex-col items-center justify-center gap-4 rounded-lg border border-border bg-surface p-8 text-center">
          <div className="rounded-full bg-error/10 p-3">
            <TrendingUp className="h-6 w-6 text-error" aria-hidden="true" />
          </div>
          <h3 className="text-lg font-semibold text-text-primary">
            Failed to load performance data
          </h3>
          <p className="max-w-sm text-sm text-text-secondary">
            Unable to retrieve performance metrics. Please try again.
          </p>
          <Button variant="primary" onClick={handleRetry}>
            <RefreshCw className="h-4 w-4" />
            Retry
          </Button>
        </div>
      </div>
    );
  }

  const performanceData: PerformanceTrend[] = data ?? [];

  /* ── Derived stats ──────────────────────────────────────── */
  const avgResponseTime =
    performanceData.length > 0
      ? Math.round(
          performanceData.reduce((s, d) => s + d.response_time_ms, 0) /
            performanceData.length,
        )
      : 0;
  const avgThroughput =
    performanceData.length > 0
      ? Math.round(
          performanceData.reduce((s, d) => s + d.throughput, 0) /
            performanceData.length,
        )
      : 0;
  const avgErrorRate =
    performanceData.length > 0
      ? performanceData.reduce((s, d) => s + d.error_rate, 0) /
        performanceData.length
      : 0;

  return (
    <div className="flex flex-col gap-lg">
      <PageHeader title="Performance Trends">
        <TimeframeFilter value={timeframe} onChange={setTimeframe} />
      </PageHeader>

      {/* Summary stat cards */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        <StatCard
          value={`${avgResponseTime}ms`}
          label="Avg Response Time"
        />
        <StatCard
          value={formatNumber(avgThroughput)}
          label="Avg Throughput (req/s)"
        />
        <StatCard
          value={`${avgErrorRate.toFixed(2)}%`}
          label="Avg Error Rate"
        />
      </div>

      {/* Response time chart */}
      {performanceData.length > 0 && (
        <section className="flex flex-col gap-4">
          <h2 className="text-lg font-semibold text-text-primary">
            Response Time Trend
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
                  <Legend />
                  <Line
                    type="monotone"
                    dataKey="response_time_ms"
                    name="Response Time (ms)"
                    stroke="var(--color-accent)"
                    strokeWidth={2}
                    dot={{ r: 4, fill: 'var(--color-accent)' }}
                  />
                </LineChart>
              </ResponsiveContainer>
            </div>
          </div>
        </section>
      )}

      {/* Throughput & Error Rate charts */}
      {performanceData.length > 0 && (
        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
          <section className="flex flex-col gap-4">
            <h2 className="text-lg font-semibold text-text-primary">
              Throughput
            </h2>
            <div className="rounded-lg border border-border bg-surface p-4">
              <div className="h-56 w-full">
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
                      dataKey="throughput"
                      name="Throughput"
                      stroke="var(--color-success)"
                      strokeWidth={2}
                      dot={{ r: 3 }}
                    />
                  </LineChart>
                </ResponsiveContainer>
              </div>
            </div>
          </section>

          <section className="flex flex-col gap-4">
            <h2 className="text-lg font-semibold text-text-primary">
              Error Rate
            </h2>
            <div className="rounded-lg border border-border bg-surface p-4">
              <div className="h-56 w-full">
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
                      unit="%"
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
                      dataKey="error_rate"
                      name="Error Rate"
                      stroke="var(--color-error)"
                      strokeWidth={2}
                      dot={{ r: 3 }}
                    />
                  </LineChart>
                </ResponsiveContainer>
              </div>
            </div>
          </section>
        </div>
      )}

      {/* Empty state */}
      {performanceData.length === 0 && (
        <p className="text-sm text-text-tertiary">
          No performance data available for the selected timeframe.
        </p>
      )}
    </div>
  );
}
