import { type ReactNode } from 'react';

/**
 * Visual status indicator with color-coded variants.
 * Maps to V2-DEEP semantic tokens: --color-success, --color-warning, etc.
 */
export type BadgeVariant =
  | 'success'
  | 'warning'
  | 'error'
  | 'info'
  | 'pending'
  | 'offline';

export type BadgeSize = 'sm' | 'md';

export interface BadgeProps {
  /** Badge text content */
  children?: ReactNode;
  /** Semantic color variant — maps to V2-DEEP token */
  variant?: BadgeVariant;
  /** Size preset */
  size?: BadgeSize;
  /** Show a colored indicator dot before the text */
  dot?: boolean;
  /** Additional CSS class names */
  className?: string;
}

/* ── Variant → fill/text styles ────────────────────────────── */
const variantStyles: Record<BadgeVariant, string> = {
  success: 'bg-success/10 text-success',
  warning: 'bg-warning/10 text-warning',
  error: 'bg-error/10 text-error',
  info: 'bg-info/10 text-info',
  pending: 'bg-pending/10 text-pending',
  offline: 'bg-offline/10 text-offline',
};

/* ── Variant → dot color ───────────────────────────────────── */
const dotStyles: Record<BadgeVariant, string> = {
  success: 'bg-success',
  warning: 'bg-warning',
  error: 'bg-error',
  info: 'bg-info',
  pending: 'bg-pending',
  offline: 'bg-offline',
};

/* ── Size presets ──────────────────────────────────────────── */
const sizeStyles: Record<BadgeSize, string> = {
  sm: 'px-1.5 py-0.5 text-xs',
  md: 'px-2.5 py-1 text-sm',
};

const baseStyles =
  'inline-flex items-center gap-1.5 rounded-full font-medium w-fit';

export function Badge({
  children,
  variant = 'info',
  size = 'md',
  dot = false,
  className = '',
}: BadgeProps) {
  return (
    <span
      className={`${baseStyles} ${variantStyles[variant]} ${sizeStyles[size]} ${className}`.trim()}
    >
      {dot && (
        <span
          className={`inline-block h-1.5 w-1.5 rounded-full ${dotStyles[variant]}`}
          aria-hidden="true"
        />
      )}
      {children}
    </span>
  );
}
