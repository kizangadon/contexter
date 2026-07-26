import { useCallback } from 'react';
import { RefreshCw, TrendingUp } from 'lucide-react';
import { useAnalyticsResources } from '@/api/hooks';
import type { ResourceUsage } from '@/api/types';
import { PageHeader } from '@/components/layout/PageHeader';
import { LoadingSkeleton } from '@/components/ui/LoadingSkeleton';
import { Button } from '@/components/ui/Button';
import { formatNumber, formatPercent } from '@/utils/formatters';

/** Resource card with progress bar */
function ResourceCard({
  label,
  value,
  percent,
}: {
  label: string;
  value: string;
  percent?: number;
}) {
  const barColor =
    percent != null
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
      {percent != null && (
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

export function AnalyticsResourcesPage() {
  const { data, isLoading, error, refetch } = useAnalyticsResources();

  const handleRetry = useCallback(() => {
    refetch();
  }, [refetch]);

  /* ── Loading state ──────────────────────────────────────── */
  if (isLoading && !error) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader
          title="Resource Usage"
          breadcrumbs={[
            { label: 'Analytics', href: '/analytics' },
            { label: 'Resources' },
          ]}
        />
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          {Array.from({ length: 4 }, (_, i) => (
            <LoadingSkeleton key={i} variant="card" />
          ))}
        </div>
      </div>
    );
  }

  /* ── Error state ────────────────────────────────────────── */
  if (error) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader
          title="Resource Usage"
          breadcrumbs={[
            { label: 'Analytics', href: '/analytics' },
            { label: 'Resources' },
          ]}
        />
        <div className="flex flex-col items-center justify-center gap-4 rounded-lg border border-border bg-surface p-8 text-center">
          <div className="rounded-full bg-error/10 p-3">
            <TrendingUp className="h-6 w-6 text-error" aria-hidden="true" />
          </div>
          <h3 className="text-lg font-semibold text-text-primary">
            Failed to load resource data
          </h3>
          <p className="max-w-sm text-sm text-text-secondary">
            Unable to retrieve resource metrics. Please try again.
          </p>
          <Button variant="primary" onClick={handleRetry}>
            <RefreshCw className="h-4 w-4" />
            Retry
          </Button>
        </div>
      </div>
    );
  }

  const resources: ResourceUsage | undefined = data;

  return (
    <div className="flex flex-col gap-lg">
      <PageHeader
        title="Resource Usage"
        breadcrumbs={[
          { label: 'Analytics', href: '/analytics' },
          { label: 'Resources' },
        ]}
      />

      {resources ? (
        <>
          {/* Resource cards grid */}
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
            <ResourceCard
              label="CPU"
              value={formatPercent(resources.cpu_percent)}
              percent={resources.cpu_percent}
            />
            <ResourceCard
              label="Memory"
              value={formatPercent(resources.memory_percent)}
              percent={resources.memory_percent}
            />
            <ResourceCard
              label="Disk"
              value={formatPercent(resources.disk_percent)}
              percent={resources.disk_percent}
            />
            <ResourceCard
              label="Active Connections"
              value={formatNumber(resources.active_connections)}
            />
          </div>

          {/* Detailed usage table */}
          <section className="flex flex-col gap-4">
            <h2 className="text-lg font-semibold text-text-primary">
              Details
            </h2>
            <div className="overflow-x-auto rounded-lg border border-border">
              <table className="w-full border-collapse text-sm">
                <thead>
                  <tr className="border-b border-border">
                    <th className="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-text-secondary">
                      Metric
                    </th>
                    <th className="px-4 py-3 text-right text-xs font-semibold uppercase tracking-wider text-text-secondary">
                      Value
                    </th>
                    <th className="px-4 py-3 text-right text-xs font-semibold uppercase tracking-wider text-text-secondary">
                      Usage
                    </th>
                  </tr>
                </thead>
                <tbody>
                  <tr className="border-b border-border hover:bg-bg-hover">
                    <td className="px-4 py-3 font-medium text-text-primary">CPU</td>
                    <td className="px-4 py-3 text-right text-text-secondary">
                      {resources.cpu_percent}%
                    </td>
                    <td className="px-4 py-3 text-right text-text-secondary">
                      {resources.cpu_percent > 80
                        ? 'High'
                        : resources.cpu_percent > 60
                          ? 'Moderate'
                          : 'Normal'}
                    </td>
                  </tr>
                  <tr className="border-b border-border hover:bg-bg-hover">
                    <td className="px-4 py-3 font-medium text-text-primary">Memory</td>
                    <td className="px-4 py-3 text-right text-text-secondary">
                      {resources.memory_percent}%
                    </td>
                    <td className="px-4 py-3 text-right text-text-secondary">
                      {resources.memory_percent > 80
                        ? 'High'
                        : resources.memory_percent > 60
                          ? 'Moderate'
                          : 'Normal'}
                    </td>
                  </tr>
                  <tr className="border-b border-border hover:bg-bg-hover">
                    <td className="px-4 py-3 font-medium text-text-primary">Disk</td>
                    <td className="px-4 py-3 text-right text-text-secondary">
                      {resources.disk_percent}%
                    </td>
                    <td className="px-4 py-3 text-right text-text-secondary">
                      {resources.disk_percent > 80
                        ? 'High'
                        : resources.disk_percent > 60
                          ? 'Moderate'
                          : 'Normal'}
                    </td>
                  </tr>
                  <tr className="hover:bg-bg-hover">
                    <td className="px-4 py-3 font-medium text-text-primary">
                      Active Connections
                    </td>
                    <td className="px-4 py-3 text-right text-text-secondary">
                      {formatNumber(resources.active_connections)}
                    </td>
                    <td className="px-4 py-3 text-right text-text-secondary">—</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </section>
        </>
      ) : (
        <p className="text-sm text-text-tertiary">
          No resource data available.
        </p>
      )}
    </div>
  );
}
