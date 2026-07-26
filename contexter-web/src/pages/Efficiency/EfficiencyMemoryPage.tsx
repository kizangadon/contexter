import { useState, useCallback } from 'react';
import { RefreshCw, TrendingUp } from 'lucide-react';
import { useEfficiencyMemory } from '@/api/hooks';
import type { EfficiencyMemory } from '@/api/types';
import { PageHeader } from '@/components/layout/PageHeader';
import { StatCard } from '@/components/ui/StatCard';
import { LoadingSkeleton } from '@/components/ui/LoadingSkeleton';
import { TimeframeFilter } from '@/components/ui/TimeframeFilter';
import { Button } from '@/components/ui/Button';
import { formatNumber, formatPercent } from '@/utils/formatters';

export function EfficiencyMemoryPage() {
  const [timeframe, setTimeframe] = useState('30d');
  const { data, isLoading, error, refetch } = useEfficiencyMemory(timeframe);

  const handleRetry = useCallback(() => {
    refetch();
  }, [refetch]);

  /* ── Loading state ──────────────────────────────────────── */
  if (isLoading && !error) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader title="Memory Usage">
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
        <PageHeader title="Memory Usage">
          <TimeframeFilter value={timeframe} onChange={setTimeframe} />
        </PageHeader>
        <div className="flex flex-col items-center justify-center gap-4 rounded-lg border border-border bg-surface p-8 text-center">
          <div className="rounded-full bg-error/10 p-3">
            <TrendingUp className="h-6 w-6 text-error" aria-hidden="true" />
          </div>
          <h3 className="text-lg font-semibold text-text-primary">
            Failed to load memory data
          </h3>
          <p className="max-w-sm text-sm text-text-secondary">
            Unable to retrieve memory metrics. Please try again.
          </p>
          <Button variant="primary" onClick={handleRetry}>
            <RefreshCw className="h-4 w-4" />
            Retry
          </Button>
        </div>
      </div>
    );
  }

  const memory: EfficiencyMemory | undefined = data;

  if (!memory) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader title="Memory Usage">
          <TimeframeFilter value={timeframe} onChange={setTimeframe} />
        </PageHeader>
        <p className="text-sm text-text-tertiary">
          No memory usage data available for the selected timeframe.
        </p>
      </div>
    );
  }

  const typeEntries = Object.entries(memory.type_distribution);
  const totalTyped = typeEntries.reduce((sum, [, count]) => sum + count, 0);

  return (
    <div className="flex flex-col gap-lg">
      <PageHeader title="Memory Usage">
        <TimeframeFilter value={timeframe} onChange={setTimeframe} />
      </PageHeader>

      {/* Summary stat cards */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        <StatCard
          value={formatNumber(memory.total_memories)}
          label="Total Memories"
        />
        <StatCard
          value={formatPercent(memory.avg_confidence * 100)}
          label="Avg Confidence"
        />
        <StatCard
          value={formatNumber(typeEntries.length)}
          label="Memory Types"
        />
      </div>

      {/* Type distribution table */}
      {typeEntries.length > 0 && (
        <section className="flex flex-col gap-4">
          <h2 className="text-lg font-semibold text-text-primary">
            Type Distribution
          </h2>
          <div className="overflow-x-auto rounded-lg border border-border">
            <table className="w-full border-collapse text-sm">
              <thead>
                <tr className="border-b border-border">
                  <th className="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-text-secondary">
                    Type
                  </th>
                  <th className="px-4 py-3 text-right text-xs font-semibold uppercase tracking-wider text-text-secondary">
                    Count
                  </th>
                  <th className="px-4 py-3 text-right text-xs font-semibold uppercase tracking-wider text-text-secondary">
                    Percentage
                  </th>
                </tr>
              </thead>
              <tbody>
                {typeEntries.map(([type, count]) => (
                  <tr
                    key={type}
                    className="border-b border-border last:border-b-0 hover:bg-bg-hover"
                  >
                    <td className="px-4 py-3 font-medium capitalize text-text-primary">
                      {type.replace(/_/g, ' ')}
                    </td>
                    <td className="px-4 py-3 text-right text-text-secondary">
                      {formatNumber(count)}
                    </td>
                    <td className="px-4 py-3 text-right text-text-secondary">
                      {totalTyped > 0
                        ? `${((count / totalTyped) * 100).toFixed(1)}%`
                        : '0%'}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>
      )}
    </div>
  );
}
