import { useId } from 'react';

export interface TimeframeOption {
  value: string;
  label: string;
}

// oxlint-disable-next-line react/only-export-components — preset values co-located with their consumer component
export const TIMEFRAME_PRESETS: TimeframeOption[] = [
  { value: '7d', label: 'Last 7 days' },
  { value: '30d', label: 'Last 30 days' },
  { value: '90d', label: 'Last 90 days' },
  { value: 'all', label: 'All time' },
  { value: 'custom', label: 'Custom' },
];

export interface TimeframeFilterProps {
  /** Currently selected value */
  value: string;
  /** Called when selection or date changes */
  onChange: (value: string) => void;
  /** Additional CSS class names */
  className?: string;
}

const selectStyles =
  'w-full rounded-md border border-border bg-bg-secondary px-3 py-2 text-sm text-text-primary outline-none transition-colors duration-150 hover:border-border-hover focus:border-accent focus:ring-1 focus:ring-accent';

export function TimeframeFilter({
  value,
  onChange,
  className = '',
}: TimeframeFilterProps) {
  const generatedId = useId();
  const selectId = `timeframe-${generatedId}`;
  const isCustom = value === 'custom';

  return (
    <div className={`flex items-center gap-2 ${className}`}>
      <select
        id={selectId}
        value={isCustom ? 'custom' : value}
        onChange={(e) => onChange(e.target.value)}
        className={selectStyles}
        aria-label="Timeframe"
      >
        {TIMEFRAME_PRESETS.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>

      {isCustom && (
        <div className="flex items-center gap-2">
          <input
            type="date"
            className={selectStyles}
            aria-label="Start date"
            onChange={(e) => onChange(`custom:${e.target.value}`)}
          />
          <span className="text-sm text-text-tertiary">to</span>
          <input
            type="date"
            className={selectStyles}
            aria-label="End date"
            onChange={(e) => onChange(`custom:${e.target.value}`)}
          />
        </div>
      )}
    </div>
  );
}
