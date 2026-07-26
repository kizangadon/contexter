import {
  type InputHTMLAttributes,
  forwardRef,
  useId,
} from 'react';
import type { LucideIcon } from 'lucide-react';

export interface InputProps
  extends Omit<InputHTMLAttributes<HTMLInputElement>, 'size'> {
  /** Lucide icon rendered on the left side */
  icon?: LucideIcon;
  /** Label text displayed above the input */
  label?: string;
  /** Helper text displayed below the input */
  helperText?: string;
  /** Error message — also applies error styling (red border) */
  error?: string;
}

/* ── Styles ────────────────────────────────────────────────── */

const sharedInputStyle =
  'w-full rounded-md border bg-transparent px-3 py-2 text-sm text-text-primary placeholder:text-text-tertiary transition-colors duration-150 outline-none';

const stateStyles = {
  default:
    'border-border hover:border-border-hover focus:border-border-active focus:ring-1 focus:ring-border-active',
  error:
    'border-error hover:border-error focus:border-error focus:ring-1 focus:ring-error',
  disabled: 'opacity-50 pointer-events-none',
};

const iconPadding = 'pl-10';

export const Input = forwardRef<HTMLInputElement, InputProps>(
  (
    { icon: Icon, label, helperText, error, disabled, className = '', ...props },
    ref,
  ) => {
    const generatedId = useId();
    const inputId = props.id ?? generatedId;

    const stateClass = error
      ? stateStyles.error
      : disabled
        ? stateStyles.disabled
        : stateStyles.default;

    const errorId = error ? `${inputId}-error` : undefined;
    const helperId = helperText && !error ? `${inputId}-helper` : undefined;

    return (
      <div className={`flex flex-col gap-1.5 ${className}`}>
        {/* ── Label ──────────────────────────────────────── */}
        {label && (
          <label
            htmlFor={inputId}
            className="text-sm font-medium text-text-primary"
          >
            {label}
          </label>
        )}

        {/* ── Input wrapper ──────────────────────────────── */}
        <div className="relative">
          {Icon && (
            <div className="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3">
              <Icon
                className="h-4 w-4 text-text-tertiary"
                aria-hidden="true"
              />
            </div>
          )}
          <input
            ref={ref}
            id={inputId}
            disabled={disabled}
            aria-invalid={error ? true : undefined}
            aria-describedby={errorId ?? helperId}
            className={`${sharedInputStyle} ${stateClass} ${Icon ? iconPadding : ''}`}
            {...props}
          />
        </div>

        {/* ── Error message ──────────────────────────────── */}
        {error && (
          <p id={errorId} className="text-xs text-error" role="alert">
            {error}
          </p>
        )}

        {/* ── Helper text (hidden when error is shown) ───── */}
        {helperText && !error && (
          <p id={helperId} className="text-xs text-text-tertiary">
            {helperText}
          </p>
        )}
      </div>
    );
  },
);

Input.displayName = 'Input';
