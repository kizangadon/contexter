import { ArrowUp, ArrowDown, Minus } from 'lucide-react';
import { LoadingSkeleton } from './LoadingSkeleton';

export interface Trend {
  direction: 'up' | 'down' | 'neutral';
  percentage: number;
}

export interface StatCardProps {
  /** Primary value displayed large */
  value: string | number;
  /** Label below the value */
  label: string;
  /** Optional trend indicator */
  trend?: Trend;
  /** Show loading skeleton placeholder */
  loading?: boolean;
  /** Additional CSS class names */
  className?: string;
}

const trendConfig: Record<
  Trend['direction'],
  { icon: typeof ArrowUp; color: string }
> = {
  up: { icon: ArrowUp, color: 'text-success' },
  down: { icon: ArrowDown, color: 'text-error' },
  neutral: { icon: Minus, color: 'text-text-tertiary' },
};

export function StatCard({
  value,
  label,
  trend,
  loading = false,
  className = '',
}: StatCardProps) {
  if (loading) {
    return (
      <div
        className={`flex flex-col gap-2 rounded-lg border border-border bg-surface p-4 ${className}`}
      >
        <LoadingSkeleton variant="text" count={1} />
        <LoadingSkeleton variant="text" count={1} />
      </div>
    );
  }

  return (
    <div
      className={`flex flex-col gap-1 rounded-lg border border-border bg-surface p-4 ${className}`}
    >
      <span
        className="text-[28px] font-bold leading-tight text-text-primary"
        style={{ fontSize: '28px' }}
      >
        {value}
      </span>
      <span className="text-sm text-text-secondary">{label}</span>
      {trend && (
        <div className="mt-1 flex items-center gap-1 text-sm font-medium">
          {trend.direction === 'up' && <ArrowUp className={`h-4 w-4 ${trendConfig[trend.direction].color}`} />}
          {trend.direction === 'down' && <ArrowDown className={`h-4 w-4 ${trendConfig[trend.direction].color}`} />}
          {trend.direction === 'neutral' && <Minus className={`h-4 w-4 ${trendConfig[trend.direction].color}`} />}
          <span className={trendConfig[trend.direction].color}>{trend.percentage}%</span>
        </div>
      )}
    </div>
  );
}
