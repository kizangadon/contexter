import { useCallback } from 'react';
import { BarChart3, RefreshCw } from 'lucide-react';
import type { CorrelationTimeline } from '@/api/types';
import {
  useCorrelationOverview,
  useCorrelationTimeline,
  useCorrelationCompare,
} from '@/api/hooks';
import { PageHeader } from '@/components/layout/PageHeader';
import { Button } from '@/components/ui/Button';
import { LoadingSkeleton } from '@/components/ui/LoadingSkeleton';
import { StatCard } from '@/components/ui/StatCard';

export function CorrelationPage() {
  const overview = useCorrelationOverview();
  const timeline = useCorrelationTimeline();
  const compare = useCorrelationCompare();

  const isLoading = overview.isLoading || timeline.isLoading || compare.isLoading;
  const isError = overview.isError || timeline.isError || compare.isError;

  const handleRetry = useCallback(() => {
    overview.refetch();
    timeline.refetch();
    compare.refetch();
  }, [overview, timeline, compare]);

  /* ── Loading ────────────────────────────────────────────── */
  if (isLoading && !isError) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader title="Correlation Analysis" />
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
          <LoadingSkeleton variant="card" />
          <LoadingSkeleton variant="card" />
          <LoadingSkeleton variant="card" />
        </div>
        <LoadingSkeleton variant="card" count={3} />
      </div>
    );
  }

  /* ── Error ──────────────────────────────────────────────── */
  if (isError) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader title="Correlation Analysis" />
        <div className="flex flex-col items-center justify-center gap-4 rounded-lg border border-border bg-surface p-8 text-center">
          <div className="rounded-full bg-error/10 p-3">
            <BarChart3 className="h-6 w-6 text-error" aria-hidden="true" />
          </div>
          <h3 className="text-lg font-semibold text-text-primary">Failed to load correlation data</h3>
          <p className="max-w-sm text-sm text-text-secondary">
            Something went wrong while fetching correlation analysis.
          </p>
          <Button variant="primary" onClick={handleRetry}>
            <RefreshCw className="h-4 w-4" />
            Retry
          </Button>
        </div>
      </div>
    );
  }

  const overviewData = overview.data;
  const timelineData: CorrelationTimeline[] = timeline.data ?? [];
  const compareData = compare.data;
  const topCorrs = overviewData?.top_correlations ?? [];
  const stats = overviewData?.dataset_stats ?? [];

  return (
    <div className="flex flex-col gap-lg">
      <PageHeader title="Correlation Analysis" />

      {/* Top Correlations Stat Cards */}
      <section className="flex flex-col gap-4">
        <h2 className="text-lg font-semibold text-text-primary">Top Correlations</h2>
        {topCorrs.length === 0 ? (
          <p className="text-sm text-text-tertiary">No correlation data available.</p>
        ) : (
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {topCorrs.map((corr, idx) => (
              <StatCard
                key={idx}
                value={corr.r.toFixed(2)}
                label={`${corr.variable_1} ↔ ${corr.variable_2}`}
                trend={
                  Math.abs(corr.r) > 0.5
                    ? { direction: corr.r > 0 ? 'up' : 'down', percentage: Math.round(Math.abs(corr.r) * 100) }
                    : undefined
                }
              />
            ))}
          </div>
        )}
      </section>

      {/* Dataset Statistics */}
      <section className="flex flex-col gap-4">
        <h2 className="text-lg font-semibold text-text-primary">Dataset Statistics</h2>
        {stats.length === 0 ? (
          <p className="text-sm text-text-tertiary">No dataset statistics available.</p>
        ) : (
          <div className="overflow-x-auto rounded-lg border border-border">
            <table className="w-full border-collapse text-sm">
              <thead>
                <tr>
                  <th className="border-b border-border px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-text-secondary">Variable</th>
                  <th className="border-b border-border px-4 py-3 text-right text-xs font-semibold uppercase tracking-wider text-text-secondary">Mean</th>
                  <th className="border-b border-border px-4 py-3 text-right text-xs font-semibold uppercase tracking-wider text-text-secondary">Std Dev</th>
                  <th className="border-b border-border px-4 py-3 text-right text-xs font-semibold uppercase tracking-wider text-text-secondary">Min</th>
                  <th className="border-b border-border px-4 py-3 text-right text-xs font-semibold uppercase tracking-wider text-text-secondary">Max</th>
                </tr>
              </thead>
              <tbody>
                {stats.map((stat) => (
                  <tr key={stat.variable} className="border-b border-border last:border-b-0 hover:bg-bg-hover">
                    <td className="px-4 py-3 font-medium text-text-primary">{stat.variable}</td>
                    <td className="px-4 py-3 text-right font-mono text-xs text-text-secondary">{stat.mean.toFixed(1)}</td>
                    <td className="px-4 py-3 text-right font-mono text-xs text-text-secondary">{stat.std.toFixed(1)}</td>
                    <td className="px-4 py-3 text-right font-mono text-xs text-text-secondary">{stat.min}</td>
                    <td className="px-4 py-3 text-right font-mono text-xs text-text-secondary">{stat.max}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>

      {/* Timeline Section */}
      <section className="flex flex-col gap-4">
        <h2 className="text-lg font-semibold text-text-primary">Trend Timeline</h2>
        {timelineData.length === 0 ? (
          <p className="text-sm text-text-tertiary">No timeline data available.</p>
        ) : (
          <div className="flex flex-col gap-3">
            {timelineData.map((point, idx) => (
              <div key={idx} className="rounded-lg border border-border bg-surface p-4">
                <div className="mb-2 text-sm font-medium text-text-primary">{point.date}</div>
                <div className="flex flex-wrap gap-4">
                  {point.correlations.map((c, ci) => (
                    <span key={ci} className="text-xs text-text-secondary">
                      {c.variable_1} ↔ {c.variable_2}: <span className="font-mono text-text-primary">{c.r.toFixed(2)}</span>
                    </span>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}
      </section>

      {/* Compare Section */}
      {compareData && (
        <section className="flex flex-col gap-4">
          <h2 className="text-lg font-semibold text-text-primary">Group Comparison</h2>
          <p className="text-sm text-text-secondary">
            Metric: <span className="font-medium text-text-primary">{compareData.metric}</span>
          </p>
          <div className="overflow-x-auto rounded-lg border border-border">
            <table className="w-full border-collapse text-sm">
              <thead>
                <tr>
                  <th className="border-b border-border px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-text-secondary">Group</th>
                  <th className="border-b border-border px-4 py-3 text-right text-xs font-semibold uppercase tracking-wider text-text-secondary">Mean</th>
                  <th className="border-b border-border px-4 py-3 text-right text-xs font-semibold uppercase tracking-wider text-text-secondary">Std Dev</th>
                  <th className="border-b border-border px-4 py-3 text-right text-xs font-semibold uppercase tracking-wider text-text-secondary">N</th>
                </tr>
              </thead>
              <tbody>
                {compareData.values.map((v) => (
                  <tr key={v.group} className="border-b border-border last:border-b-0 hover:bg-bg-hover">
                    <td className="px-4 py-3 font-medium text-text-primary">{v.group}</td>
                    <td className="px-4 py-3 text-right font-mono text-xs text-text-secondary">{v.mean.toFixed(1)}</td>
                    <td className="px-4 py-3 text-right font-mono text-xs text-text-secondary">{v.std.toFixed(1)}</td>
                    <td className="px-4 py-3 text-right font-mono text-xs text-text-secondary">{v.n}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <p className="text-xs text-text-tertiary">
            {compareData.test.type} test: statistic = {compareData.test.statistic.toFixed(2)},{' '}
            p = {compareData.test.p_value.toFixed(4)} —{' '}
            {compareData.test.significant ? (
              <span className="font-medium text-success">Significant</span>
            ) : (
              <span className="font-medium text-text-tertiary">Not significant</span>
            )}
          </p>
        </section>
      )}
    </div>
  );
}
