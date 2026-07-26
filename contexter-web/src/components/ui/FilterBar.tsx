import { useId } from 'react';
import { Input } from './Input';

export interface FilterOption {
  value: string;
  label: string;
}

export interface FilterDef {
  /** Unique key for this filter */
  key: string;
  /** Display label */
  label: string;
  /** Available options */
  options: FilterOption[];
  /** Currently selected value */
  value: string;
  /** Called when selection changes */
  onChange: (value: string) => void;
}

export interface FilterBarProps {
  /** Array of filter definitions */
  filters: FilterDef[];
  /** Optional search callback — renders a search input when provided */
  onSearch?: (query: string) => void;
  /** Placeholder text for the search input */
  searchPlaceholder?: string;
  /** Additional CSS class names */
  className?: string;
}

const selectStyles =
  'rounded-md border border-border bg-bg-secondary px-3 py-2 text-sm text-text-primary outline-none transition-colors duration-150 hover:border-border-hover focus:border-accent focus:ring-1 focus:ring-accent';

export function FilterBar({
  filters,
  onSearch,
  searchPlaceholder = 'Search…',
  className = '',
}: FilterBarProps) {
  const generatedId = useId();

  return (
    <div
      className={`flex flex-wrap items-center gap-3 ${className}`}
    >
      {filters.map((filter) => {
        const selectId = `filter-${filter.key}-${generatedId}`;
        return (
          <div key={filter.key} className="flex items-center gap-2">
            <label
              htmlFor={selectId}
              className="whitespace-nowrap text-sm font-medium text-text-secondary"
            >
              {filter.label}
            </label>
            <select
              id={selectId}
              value={filter.value}
              onChange={(e) => filter.onChange(e.target.value)}
              className={selectStyles}
              aria-label={filter.label}
            >
              {filter.options.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
          </div>
        );
      })}

      {onSearch && (
        <div className="ml-auto">
          <Input
            placeholder={searchPlaceholder}
            onChange={(e) => onSearch(e.target.value)}
            aria-label="Search"
          />
        </div>
      )}
    </div>
  );
}
