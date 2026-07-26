import { useCallback } from 'react';
import { useParams, Link } from 'react-router';
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
import { useAnalyticsModelDetail } from '@/api/hooks';
import { PageHeader } from '@/components/layout/PageHeader';
import { LoadingSkeleton } from '@/components/ui/LoadingSkeleton';
import { EmptyState } from '@/components/ui/EmptyState';
import { Button } from '@/components/ui/Button';
import { formatCurrency, formatNumber } from '@/utils/formatters';

export function AnalyticsModelDetailPage() {
  const { id } = useParams<{ id: string }>();
  const { data, isLoading, error, refetch } = useAnalyticsModelDetail(id ?? '');

  const handleRetry = useCallback(() => {
    refetch();
  }, [refetch]);

  /* ── Missing id ─────────────────────────────────────────── */
  if (!id) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader
          title="Model Details"
          breadcrumbs={[
            { label: 'Analytics', href: '/analytics' },
            { label: 'Costs', href: '/analytics/costs' },
            { label: 'Model' },
          ]}
        />
        <EmptyState
          title="No model specified"
          message="Select a model from the cost analytics page to view details."
        />
      </div>
    );
  }

  /* ── Loading state ──────────────────────────────────────── */
  if (isLoading && !error) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader
          title={id}
          breadcrumbs={[
            { label: 'Analytics', href: '/analytics' },
            { label: 'Costs', href: '/analytics/costs' },
            { label: id },
          ]}
        />
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          {Array.from({ length: 4 }, (_, i) => (
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
        <PageHeader
          title={id}
          breadcrumbs={[
            { label: 'Analytics', href: '/analytics' },
            { label: 'Costs', href: '/analytics/costs' },
            { label: id },
          ]}
        />
        <div className="flex flex-col items-center justify-center gap-4 rounded-lg border border-border bg-surface p-8 text-center">
          <div className="rounded-full bg-error/10 p-3">
            <TrendingUp className="h-6 w-6 text-error" aria-hidden="true" />
          </div>
          <h3 className="text-lg font-semibold text-text-primary">
            Failed to load model data
          </h3>
          <p className="max-w-sm text-sm text-text-secondary">
            Unable to retrieve details for model "{id}". Please try again.
          </p>
          <Button variant="primary" onClick={handleRetry}>
            <RefreshCw className="h-4 w-4" />
            Retry
          </Button>
        </div>
      </div>
    );
  }

  if (!data) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader
          title={id}
          breadcrumbs={[
            { label: 'Analytics', href: '/analytics' },
            { label: 'Costs', href: '/analytics/costs' },
            { label: id },
          ]}
        />
        <EmptyState
          title="Model not found"
          message={`No data available for model "${id}".`}
          action={
            <Link
              to="/analytics/costs"
              className="text-sm font-medium text-accent transition-colors hover:text-accent-hover"
            >
              &larr; Back to Cost Analytics
            </Link>
          }
        />
      </div>
    );
  }

  /* ── Data state ─────────────────────────────────────────── */
  return (
    <div className="flex flex-col gap-lg">
      <PageHeader
        title={data.model}
        breadcrumbs={[
          { label: 'Analytics', href: '/analytics' },
          { label: 'Costs', href: '/analytics/costs' },
          { label: data.model },
        ]}
      />

      {/* Summary stat cards */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <div className="flex flex-col gap-1 rounded-lg border border-border bg-surface p-4">
          <span className="text-xs font-medium uppercase tracking-wider text-text-secondary">
            Total Cost
          </span>
          <span className="text-2xl font-bold text-text-primary">
            {formatCurrency(data.total_cost)}
          </span>
        </div>
        <div className="flex flex-col gap-1 rounded-lg border border-border bg-surface p-4">
          <span className="text-xs font-medium uppercase tracking-wider text-text-secondary">
            Total Tokens
          </span>
          <span className="text-2xl font-bold text-text-primary">
            {formatNumber(data.total_tokens)}
          </span>
        </div>
        <div className="flex flex-col gap-1 rounded-lg border border-border bg-surface p-4">
          <span className="text-xs font-medium uppercase tracking-wider text-text-secondary">
            Input Tokens
          </span>
          <span className="text-2xl font-bold text-text-primary">
            {formatNumber(data.input_tokens)}
          </span>
        </div>
        <div className="flex flex-col gap-1 rounded-lg border border-border bg-surface p-4">
          <span className="text-xs font-medium uppercase tracking-wider text-text-secondary">
            Cost per Token
          </span>
          <span className="text-2xl font-bold text-text-primary">
            {formatCurrency(data.avg_cost_per_token)}
          </span>
        </div>
      </div>

      {/* Daily cost breakdown chart */}
      {data.daily_breakdown.length > 0 && (
        <section className="flex flex-col gap-4">
          <h2 className="text-lg font-semibold text-text-primary">
            Daily Cost Trend
          </h2>
          <div className="rounded-lg border border-border bg-surface p-4">
            <div className="h-64 w-full">
              <ResponsiveContainer width="100%" height="100%">
                <LineChart data={data.daily_breakdown}>
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
                    unit="$"
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
                    dataKey="cost"
                    name="Cost"
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

      {/* Daily breakdown table */}
      {data.daily_breakdown.length > 0 && (
        <section className="flex flex-col gap-4">
          <h2 className="text-lg font-semibold text-text-primary">
            Daily Breakdown
          </h2>
          <div className="overflow-x-auto rounded-lg border border-border">
            <table className="w-full border-collapse text-sm">
              <thead>
                <tr className="border-b border-border">
                  <th className="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-text-secondary">
                    Date
                  </th>
                  <th className="px-4 py-3 text-right text-xs font-semibold uppercase tracking-wider text-text-secondary">
                    Tokens
                  </th>
                  <th className="px-4 py-3 text-right text-xs font-semibold uppercase tracking-wider text-text-secondary">
                    Cost
                  </th>
                </tr>
              </thead>
              <tbody>
                {data.daily_breakdown.map((day) => (
                  <tr
                    key={day.date}
                    className="border-b border-border last:border-b-0 hover:bg-bg-hover"
                  >
                    <td className="px-4 py-3 font-medium text-text-primary">
                      {day.date}
                    </td>
                    <td className="px-4 py-3 text-right text-text-secondary">
                      {formatNumber(day.tokens)}
                    </td>
                    <td className="px-4 py-3 text-right text-text-secondary">
                      {formatCurrency(day.cost)}
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
