import { type ButtonHTMLAttributes, type ReactNode } from 'react';

export interface ToggleChipProps
  extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'children'> {
  /** Whether the chip is in active/selected state */
  active?: boolean;
  /** Click handler */
  onClick: () => void;
  /** Label text */
  children: ReactNode;
}

const baseStyles =
  'inline-flex items-center rounded-full px-4 py-1.5 text-sm font-medium transition-colors duration-150 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent';

const activeStyles = 'bg-accent text-text-inverse';
const inactiveStyles = 'bg-bg-tertiary text-text-secondary hover:bg-bg-hover';

export function ToggleChip({
  active = false,
  onClick,
  children,
  className = '',
  ...props
}: ToggleChipProps) {
  return (
    <button
      type="button"
      className={`${baseStyles} ${active ? activeStyles : inactiveStyles} ${className}`.trim()}
      onClick={onClick}
      aria-pressed={active}
      {...props}
    >
      {children}
    </button>
  );
}
