import { useState, useCallback } from 'react';
import { RefreshCw, TrendingUp } from 'lucide-react';
import { useEfficiencyCorrelation } from '@/api/hooks';
import type { CorrelationMatrix } from '@/api/types';
import { PageHeader } from '@/components/layout/PageHeader';
import { StatCard } from '@/components/ui/StatCard';
import { LoadingSkeleton } from '@/components/ui/LoadingSkeleton';
import { EmptyState } from '@/components/ui/EmptyState';
import { TimeframeFilter } from '@/components/ui/TimeframeFilter';
import { Button } from '@/components/ui/Button';

export function EfficiencyCorrelationPage() {
  const [timeframe, setTimeframe] = useState('30d');
  const { data, isLoading, error, refetch } = useEfficiencyCorrelation(timeframe);

  const handleRetry = useCallback(() => {
    refetch();
  }, [refetch]);

  /* ── Loading state ──────────────────────────────────────── */
  if (isLoading && !error) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader title="Correlation Matrix">
          <TimeframeFilter value={timeframe} onChange={setTimeframe} />
        </PageHeader>
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {Array.from({ length: 2 }, (_, i) => (
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
        <PageHeader title="Correlation Matrix">
          <TimeframeFilter value={timeframe} onChange={setTimeframe} />
        </PageHeader>
        <div className="flex flex-col items-center justify-center gap-4 rounded-lg border border-border bg-surface p-8 text-center">
          <div className="rounded-full bg-error/10 p-3">
            <TrendingUp className="h-6 w-6 text-error" aria-hidden="true" />
          </div>
          <h3 className="text-lg font-semibold text-text-primary">
            Failed to load correlation data
          </h3>
          <p className="max-w-sm text-sm text-text-secondary">
            Unable to retrieve correlation matrix. Please try again.
          </p>
          <Button variant="primary" onClick={handleRetry}>
            <RefreshCw className="h-4 w-4" />
            Retry
          </Button>
        </div>
      </div>
    );
  }

  const correlation: CorrelationMatrix | undefined = data;

  if (!correlation) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader title="Correlation Matrix">
          <TimeframeFilter value={timeframe} onChange={setTimeframe} />
        </PageHeader>
        <EmptyState
          title="No correlation data available"
          message="Correlation data will appear here once sufficient metrics have been collected."
        />
      </div>
    );
  }

  const { variables, correlations } = correlation;

  const rowCount = correlations.length;
  const hasMatrix = variables.length > 0 && rowCount > 0;

  /** Color strength based on absolute correlation value */
  function correlationColor(value: number, isSelf: boolean): string {
    const abs = Math.abs(value);
    if (isSelf) return 'font-bold';
    if (abs > 0.5) return 'text-accent';
    if (abs > 0.3) return 'text-text-primary';
    return 'text-text-tertiary';
  }

  return (
    <div className="flex flex-col gap-lg">
      <PageHeader title="Correlation Matrix">
        <TimeframeFilter value={timeframe} onChange={setTimeframe} />
      </PageHeader>

      {/* Summary stats */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        <StatCard
          value={variables.length}
          label="Variables"
        />
        <StatCard
          value={correlations.length > 0 ? `${correlations.length}×${correlations.length}` : '—'}
          label="Matrix Size"
        />
      </div>

      {/* Correlation matrix table */}
      {hasMatrix && (
        <section className="flex flex-col gap-4">
          <h2 className="text-lg font-semibold text-text-primary">
            Correlation Coefficients
          </h2>
          <div className="overflow-x-auto rounded-lg border border-border">
            <table className="w-full border-collapse text-sm">
              <thead>
                <tr>
                  <th className="border-b border-border px-3 py-2 text-left text-xs font-semibold uppercase tracking-wider text-text-secondary">
                    Variable
                  </th>
                  {variables.map((v) => (
                    <th
                      key={v}
                      className="border-b border-border px-3 py-2 text-right text-xs font-semibold uppercase tracking-wider text-text-secondary"
                    >
                      {v}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {variables.map((variable, rowIdx) => {
                  const row = correlations[rowIdx];
                  return (
                    <tr
                      key={variable}
                      className="border-b border-border last:border-b-0 hover:bg-bg-hover"
                    >
                      <td className="px-3 py-2 font-medium text-text-primary">
                        {variable}
                      </td>
                      {row?.map((value, colIdx) => {
                        const isSelf = rowIdx === colIdx;

                        return (
                          <td
                            key={`${variable}-${variables[colIdx]}`}
                            className={`px-3 py-2 text-right font-mono text-xs ${correlationColor(value, isSelf)}`}
                          >
                            {value.toFixed(2)}
                          </td>
                        );
                      })}
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>

          {/* Legend */}
          <div className="flex items-center gap-4 text-xs text-text-secondary">
            <span>
              <span className="font-bold text-accent">r &gt; 0.5</span> Strong
            </span>
            <span>
              <span className="font-bold text-text-primary">r &gt; 0.3</span> Moderate
            </span>
            <span>
              <span className="font-bold text-text-tertiary">r ≤ 0.3</span> Weak
            </span>
          </div>
        </section>
      )}
    </div>
  );
}
