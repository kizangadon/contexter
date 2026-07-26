import { useState, useCallback } from 'react';
import { RefreshCw, Server, TrendingUp } from 'lucide-react';
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
  useAnalyticsServices,
  useAnalyticsModelDetail,
} from '@/api/hooks';
import type { ServiceStatus, ModelCostDetail } from '@/api/types';
import { PageHeader } from '@/components/layout/PageHeader';
import { Badge } from '@/components/ui/Badge';
import { LoadingSkeleton } from '@/components/ui/LoadingSkeleton';
import { EmptyState } from '@/components/ui/EmptyState';
import { Button } from '@/components/ui/Button';
import { formatCurrency, formatNumber } from '@/utils/formatters';

/* ─── Status badge variant mapping ─────────────────────────── */
type ServiceStatusVariant = 'success' | 'warning' | 'error' | 'info';

function statusToVariant(status: string): ServiceStatusVariant {
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

export function AnalyticsModelsPage() {
  const services = useAnalyticsServices();
  const [selectedModel, setSelectedModel] = useState<string>('');
  const modelDetail = useAnalyticsModelDetail(selectedModel);

  /* ── Derived state ─────────────────────────────────────── */
  const isLoading = services.isLoading;
  const isError = services.isError;

  const servicesData: ServiceStatus[] = services.data ?? [];

  /* ── Retry handler ──────────────────────────────────────── */
  const handleRetry = useCallback(() => {
    services.refetch();
    if (selectedModel) {
      modelDetail.refetch();
    }
  }, [services, modelDetail, selectedModel]);

  /* ── Model detail handler ───────────────────────────────── */
  const handleModelClick = useCallback(
    (modelName: string) => {
      setSelectedModel((prev) => (prev === modelName ? '' : modelName));
    },
    [],
  );

  /* ── Render loading state ───────────────────────────────── */
  if (isLoading && !isError) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader
          title="Model Analytics"
          breadcrumbs={[
            { label: 'Analytics', href: '/analytics' },
            { label: 'Models' },
          ]}
        />
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {Array.from({ length: 5 }, (_, i) => (
            <LoadingSkeleton key={i} variant="card" />
          ))}
        </div>
      </div>
    );
  }

  /* ── Render error state ─────────────────────────────────── */
  if (isError) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader
          title="Model Analytics"
          breadcrumbs={[
            { label: 'Analytics', href: '/analytics' },
            { label: 'Models' },
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
            Something went wrong while fetching model analytics. Please try
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
  const noServices = servicesData.length === 0;

  return (
    <div className="flex flex-col gap-lg">
      {/* Header + Breadcrumbs */}
      <PageHeader
        title="Model Analytics"
        breadcrumbs={[
          { label: 'Analytics', href: '/analytics' },
          { label: 'Models' },
        ]}
      />

      {/* Service Status Cards */}
      {noServices ? (
        <div className="rounded-lg border border-border">
          <EmptyState
            icon={Server}
            title="No services available"
            message="Service status information will appear here once services are registered."
          />
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {servicesData.map((svc) => {
            const isSelected = selectedModel === svc.name;
            return (
              <button
                key={svc.name}
                type="button"
                onClick={() => handleModelClick(svc.name)}
                className={`flex flex-col gap-3 rounded-lg border p-4 text-left transition-colors ${
                  isSelected
                    ? 'border-accent bg-accent/5'
                    : 'border-border bg-surface hover:border-accent/30 hover:bg-surface-hover'
                }`}
              >
                <div className="flex items-center justify-between">
                  <span className="font-semibold text-text-primary">
                    {svc.name}
                  </span>
                  <Badge variant={statusToVariant(svc.status)} size="sm" dot>
                    {svc.status}
                  </Badge>
                </div>
                <div className="flex items-center gap-4 text-xs text-text-secondary">
                  <span>Uptime: {svc.uptime_percent}%</span>
                  <span>Latency: {svc.latency_ms}ms</span>
                </div>
              </button>
            );
          })}
        </div>
      )}

      {/* Model Detail Section */}
      {selectedModel && modelDetail.data && (
        <ModelDetailSection data={modelDetail.data} />
      )}
    </div>
  );
}

/* ─── Model Detail Section ─────────────────────────────────── */

function ModelDetailSection({ data }: { data: ModelCostDetail }) {
  return (
    <section className="flex flex-col gap-4">
      <h2 className="text-lg font-semibold text-text-primary">
        {data.model} — Cost Breakdown
      </h2>

      {/* Summary stats */}
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
            Output Tokens
          </span>
          <span className="text-2xl font-bold text-text-primary">
            {formatNumber(data.output_tokens)}
          </span>
        </div>
      </div>

      {/* Daily cost trend chart */}
      <div className="rounded-lg border border-border bg-surface p-4">
        <h3 className="mb-3 text-sm font-semibold text-text-primary">
          Daily Cost Trend
        </h3>
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
  );
}
