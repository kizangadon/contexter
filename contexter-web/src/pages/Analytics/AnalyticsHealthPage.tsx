import { useCallback } from 'react';
import { RefreshCw, TrendingUp } from 'lucide-react';
import { useAnalyticsHealth } from '@/api/hooks';
import type { HealthStatus } from '@/api/hooks';
import { PageHeader } from '@/components/layout/PageHeader';
import { Badge } from '@/components/ui/Badge';
import { LoadingSkeleton } from '@/components/ui/LoadingSkeleton';
import { Button } from '@/components/ui/Button';

/** Health status mapping to badge variant */
function statusToVariant(status: string): 'success' | 'warning' | 'error' | 'info' {
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

/** Format uptime seconds into a human-readable string */
function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  const parts: string[] = [];
  if (days > 0) parts.push(`${days}d`);
  if (hours > 0) parts.push(`${hours}h`);
  parts.push(`${mins}m`);
  return parts.join(' ');
}

export function AnalyticsHealthPage() {
  const { data, isLoading, error, refetch } = useAnalyticsHealth();

  const handleRetry = useCallback(() => {
    refetch();
  }, [refetch]);

  /* ── Loading state ──────────────────────────────────────── */
  if (isLoading && !error) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader
          title="System Health"
          breadcrumbs={[
            { label: 'Analytics', href: '/analytics' },
            { label: 'Health' },
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
          title="System Health"
          breadcrumbs={[
            { label: 'Analytics', href: '/analytics' },
            { label: 'Health' },
          ]}
        />
        <div className="flex flex-col items-center justify-center gap-4 rounded-lg border border-border bg-surface p-8 text-center">
          <div className="rounded-full bg-error/10 p-3">
            <TrendingUp className="h-6 w-6 text-error" aria-hidden="true" />
          </div>
          <h3 className="text-lg font-semibold text-text-primary">
            Failed to load health data
          </h3>
          <p className="max-w-sm text-sm text-text-secondary">
            Unable to retrieve system health information. Please try again.
          </p>
          <Button variant="primary" onClick={handleRetry}>
            <RefreshCw className="h-4 w-4" />
            Retry
          </Button>
        </div>
      </div>
    );
  }

  /* ── Empty state ────────────────────────────────────────── */
  if (!data) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader
          title="System Health"
          breadcrumbs={[
            { label: 'Analytics', href: '/analytics' },
            { label: 'Health' },
          ]}
        />
        <p className="text-sm text-text-tertiary">No health data available.</p>
      </div>
    );
  }

  /* ── Data state ─────────────────────────────────────────── */
  const healthData: HealthStatus | undefined = data;

  const services: { name: string; status: string }[] = [];
  if (healthData.services) {
    for (const [name, status] of Object.entries(healthData.services)) {
      services.push({ name, status: status as string });
    }
  }

  return (
    <div className="flex flex-col gap-lg">
      <PageHeader
        title="System Health"
        breadcrumbs={[
          { label: 'Analytics', href: '/analytics' },
          { label: 'Health' },
        ]}
      />

      {/* System status overview */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        <div className="flex flex-col gap-1 rounded-lg border border-border bg-surface p-4">
          <span className="text-sm text-text-secondary">System Status</span>
          <div className="flex items-center gap-2">
            <Badge variant={statusToVariant(healthData.status)} dot>
              {healthData.status}
            </Badge>
          </div>
        </div>

        {healthData.uptime_seconds != null && (
          <div className="flex flex-col gap-1 rounded-lg border border-border bg-surface p-4">
            <span className="text-sm text-text-secondary">Uptime</span>
            <span className="text-2xl font-bold text-text-primary">
              {formatUptime(healthData.uptime_seconds)}
            </span>
          </div>
        )}

        {healthData.version && (
          <div className="flex flex-col gap-1 rounded-lg border border-border bg-surface p-4">
            <span className="text-sm text-text-secondary">Version</span>
            <span className="text-2xl font-bold text-text-primary">
              {healthData.version}
            </span>
          </div>
        )}
      </div>

      {/* Service status indicators */}
      {services.length > 0 && (
        <section className="flex flex-col gap-4">
          <h2 className="text-lg font-semibold text-text-primary">
            Services
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
      )}
    </div>
  );
}
