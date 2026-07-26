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
} from 'recharts';
import { useEfficiencyTokens } from '@/api/hooks';
import type { EfficiencyTokens } from '@/api/types';
import { PageHeader } from '@/components/layout/PageHeader';
import { StatCard } from '@/components/ui/StatCard';
import { LoadingSkeleton } from '@/components/ui/LoadingSkeleton';
import { TimeframeFilter } from '@/components/ui/TimeframeFilter';
import { Button } from '@/components/ui/Button';
import { formatNumber } from '@/utils/formatters';

export function EfficiencyTokensPage() {
  const [timeframe, setTimeframe] = useState('30d');
  const { data, isLoading, error, refetch } = useEfficiencyTokens(timeframe);

  const handleRetry = useCallback(() => {
    refetch();
  }, [refetch]);

  /* ── Loading state ──────────────────────────────────────── */
  if (isLoading && !error) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader title="Token Usage">
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
        <PageHeader title="Token Usage">
          <TimeframeFilter value={timeframe} onChange={setTimeframe} />
        </PageHeader>
        <div className="flex flex-col items-center justify-center gap-4 rounded-lg border border-border bg-surface p-8 text-center">
          <div className="rounded-full bg-error/10 p-3">
            <TrendingUp className="h-6 w-6 text-error" aria-hidden="true" />
          </div>
          <h3 className="text-lg font-semibold text-text-primary">
            Failed to load token data
          </h3>
          <p className="max-w-sm text-sm text-text-secondary">
            Unable to retrieve token consumption data. Please try again.
          </p>
          <Button variant="primary" onClick={handleRetry}>
            <RefreshCw className="h-4 w-4" />
            Retry
          </Button>
        </div>
      </div>
    );
  }

  const tokens: EfficiencyTokens | undefined = data;
  const modelEntries = tokens ? Object.entries(tokens.by_model) : [];

  return (
    <div className="flex flex-col gap-lg">
      <PageHeader title="Token Usage">
        <TimeframeFilter value={timeframe} onChange={setTimeframe} />
      </PageHeader>

      {tokens ? (
        <>
          {/* Summary stat cards */}
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
            <StatCard
              value={formatNumber(tokens.total_tokens)}
              label="Total Tokens"
            />
            <StatCard
              value={formatNumber(Math.round(tokens.avg_per_session))}
              label="Avg per Session"
            />
            <StatCard
              value={formatNumber(modelEntries.length)}
              label="Models Used"
            />
          </div>

          {/* Daily token trend chart */}
          {tokens.daily.length > 0 && (
            <section className="flex flex-col gap-4">
              <h2 className="text-lg font-semibold text-text-primary">
                Daily Token Consumption
              </h2>
              <div className="rounded-lg border border-border bg-surface p-4">
                <div className="h-64 w-full">
                  <ResponsiveContainer width="100%" height="100%">
                    <LineChart data={tokens.daily}>
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
                        dataKey="tokens"
                        name="Tokens"
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
          )}

          {/* By-model breakdown */}
          {modelEntries.length > 0 && (
            <section className="flex flex-col gap-4">
              <h2 className="text-lg font-semibold text-text-primary">
                Tokens by Model
              </h2>
              <div className="overflow-x-auto rounded-lg border border-border">
                <table className="w-full border-collapse text-sm">
                  <thead>
                    <tr className="border-b border-border">
                      <th className="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-text-secondary">
                        Model
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
                    {modelEntries.map(([model, count]) => (
                      <tr
                        key={model}
                        className="border-b border-border last:border-b-0 hover:bg-bg-hover"
                      >
                        <td className="px-4 py-3 font-medium text-text-primary">
                          {model}
                        </td>
                        <td className="px-4 py-3 text-right text-text-secondary">
                          {formatNumber(count)}
                        </td>
                        <td className="px-4 py-3 text-right text-text-secondary">
                          {tokens.total_tokens > 0
                            ? `${((count / tokens.total_tokens) * 100).toFixed(1)}%`
                            : '0%'}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </section>
          )}
        </>
      ) : (
        <p className="text-sm text-text-tertiary">
          No token usage data available for the selected timeframe.
        </p>
      )}
    </div>
  );
}
