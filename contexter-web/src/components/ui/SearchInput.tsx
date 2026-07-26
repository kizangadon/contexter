import { type InputHTMLAttributes, forwardRef, useId } from 'react';
import { Search, X } from 'lucide-react';

export interface SearchInputProps
  extends Omit<InputHTMLAttributes<HTMLInputElement>, 'size'> {
  /** Placeholder text */
  placeholder?: string;
  /** Called when the clear button is clicked */
  onClear?: () => void;
  /** Keyboard shortcut hint displayed inside the input */
  shortcut?: string;
}

const inputStyles =
  'w-full rounded-md border border-border bg-bg-secondary pl-10 pr-10 py-2 text-sm text-text-primary placeholder:text-text-tertiary transition-colors duration-150 outline-none hover:border-border-hover focus:border-accent focus:ring-1 focus:ring-accent';

export const SearchInput = forwardRef<HTMLInputElement, SearchInputProps>(
  ({ placeholder = 'Search...', onClear, shortcut, value, onChange, className = '', ...props }, ref) => {
    const generatedId = useId();
    const inputId = `search-${generatedId}`;

    const hasValue = typeof value === 'string' && value.length > 0;

    return (
      <div className={`relative ${className}`}>
        {/* Search icon */}
        <div className="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3">
          <Search className="h-4 w-4 text-text-tertiary" aria-hidden="true" />
        </div>

        <input
          ref={ref}
          id={inputId}
          type="search"
          value={value}
          onChange={onChange}
          placeholder={placeholder}
          className={inputStyles}
          aria-label="Search"
          {...props}
        />

        {/* Right side: clear button + shortcut hint */}
        <div className="absolute inset-y-0 right-0 flex items-center gap-1 pr-3">
          {hasValue && (
            <button
              type="button"
              onClick={onClear}
              className="flex h-5 w-5 items-center justify-center rounded text-text-tertiary transition-colors hover:text-text-primary"
              aria-label="Clear search"
            >
              <X className="h-3.5 w-3.5" />
            </button>
          )}
          {!hasValue && shortcut && (
            <kbd className="hidden rounded border border-border bg-bg-primary px-1.5 py-0.5 text-[10px] font-medium text-text-tertiary sm:inline-block">
              {shortcut}
            </kbd>
          )}
        </div>
      </div>
    );
  },
);

SearchInput.displayName = 'SearchInput';
