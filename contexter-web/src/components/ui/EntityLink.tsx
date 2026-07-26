import { type ReactNode } from 'react';
import { Link } from 'react-router';

export type EntityType = 'session' | 'memory' | 'agent' | 'skill';

export interface EntityLinkProps {
  /** Route path to link to */
  to: string;
  /** Link content */
  children: ReactNode;
  /** Entity type — renders a colored dot indicator */
  type?: EntityType;
  /** Additional CSS class names */
  className?: string;
}

const dotColors: Record<EntityType, string> = {
  session: 'bg-accent',
  memory: 'bg-success',
  agent: 'bg-info',
  skill: 'bg-warning',
};

export function EntityLink({
  to,
  children,
  type,
  className = '',
}: EntityLinkProps) {
  return (
    <Link
      to={to}
      className={`inline-flex items-center gap-1.5 text-accent hover:underline ${className}`.trim()}
    >
      {type && (
        <span
          className={`inline-block h-2 w-2 rounded-full ${dotColors[type]}`}
          aria-hidden="true"
        />
      )}
      {children}
    </Link>
  );
}
