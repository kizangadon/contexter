import { useCallback } from 'react';
import { RefreshCw, TrendingUp } from 'lucide-react';
import { useAnalyticsServices } from '@/api/hooks';
import type { ServiceStatus } from '@/api/types';
import { PageHeader } from '@/components/layout/PageHeader';
import { Badge } from '@/components/ui/Badge';
import { LoadingSkeleton } from '@/components/ui/LoadingSkeleton';
import { EmptyState } from '@/components/ui/EmptyState';
import { Button } from '@/components/ui/Button';

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

export function AnalyticsServicesPage() {
  const { data, isLoading, error, refetch } = useAnalyticsServices();

  const handleRetry = useCallback(() => {
    refetch();
  }, [refetch]);

  /* ── Loading state ──────────────────────────────────────── */
  if (isLoading && !error) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader
          title="Service Status"
          breadcrumbs={[
            { label: 'Analytics', href: '/analytics' },
            { label: 'Services' },
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

  /* ── Error state ────────────────────────────────────────── */
  if (error) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader
          title="Service Status"
          breadcrumbs={[
            { label: 'Analytics', href: '/analytics' },
            { label: 'Services' },
          ]}
        />
        <div className="flex flex-col items-center justify-center gap-4 rounded-lg border border-border bg-surface p-8 text-center">
          <div className="rounded-full bg-error/10 p-3">
            <TrendingUp className="h-6 w-6 text-error" aria-hidden="true" />
          </div>
          <h3 className="text-lg font-semibold text-text-primary">
            Failed to load service data
          </h3>
          <p className="max-w-sm text-sm text-text-secondary">
            Unable to retrieve service status information. Please try again.
          </p>
          <Button variant="primary" onClick={handleRetry}>
            <RefreshCw className="h-4 w-4" />
            Retry
          </Button>
        </div>
      </div>
    );
  }

  const services: ServiceStatus[] = data ?? [];

  return (
    <div className="flex flex-col gap-lg">
      <PageHeader
        title="Service Status"
        breadcrumbs={[
          { label: 'Analytics', href: '/analytics' },
          { label: 'Services' },
        ]}
      />

      {services.length === 0 ? (
        <EmptyState
          title="No services available"
          message="Service status information will appear here once services are registered."
        />
      ) : (
        <>
          {/* Summary stats */}
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
            <div className="flex flex-col gap-1 rounded-lg border border-border bg-surface p-4">
              <span className="text-sm text-text-secondary">Total Services</span>
              <span className="text-2xl font-bold text-text-primary">
                {services.length}
              </span>
            </div>
            <div className="flex flex-col gap-1 rounded-lg border border-border bg-surface p-4">
              <span className="text-sm text-text-secondary">Healthy</span>
              <span className="text-2xl font-bold text-success">
                {services.filter((s) => s.status === 'healthy').length}
              </span>
            </div>
            <div className="flex flex-col gap-1 rounded-lg border border-border bg-surface p-4">
              <span className="text-sm text-text-secondary">Degraded / Down</span>
              <span className="text-2xl font-bold text-error">
                {services.filter((s) => s.status !== 'healthy').length}
              </span>
            </div>
          </div>

          {/* Service cards */}
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {services.map((svc) => (
              <div
                key={svc.name}
                className="flex flex-col gap-3 rounded-lg border border-border bg-surface p-4"
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
                <div className="text-xs text-text-tertiary">
                  Last checked: {new Date(svc.last_checked).toLocaleString()}
                </div>
              </div>
            ))}
          </div>
        </>
      )}
    </div>
  );
}
