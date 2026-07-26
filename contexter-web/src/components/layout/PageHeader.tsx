import { type ReactNode } from 'react';
import type { Breadcrumb } from './TopBar';

interface PageHeaderProps {
  /** Page title rendered as h1 */
  title: string;
  /** Optional breadcrumb trail */
  breadcrumbs?: Breadcrumb[];
  /** Optional action buttons rendered on the right */
  children?: ReactNode;
}

export function PageHeader({ title, breadcrumbs, children }: PageHeaderProps) {
  return (
    <div className="mb-lg">
      {/* Breadcrumbs (optional) */}
      {breadcrumbs && breadcrumbs.length > 0 && (
        <nav aria-label="Breadcrumb" className="mb-2">
          <ol className="flex items-center gap-1.5 text-sm">
            {breadcrumbs.map((crumb, index) => {
              const isLast = index === breadcrumbs.length - 1;

              return (
                <li key={`${crumb.label}-${index}`} className="flex items-center gap-1.5">
                  {index > 0 && (
                    <span className="text-text-tertiary" aria-hidden="true">
                      /
                    </span>
                  )}
                  {crumb.href && !isLast ? (
                    <a
                      href={crumb.href}
                      className="text-text-secondary transition-colors duration-150 hover:text-text-primary"
                    >
                      {crumb.label}
                    </a>
                  ) : (
                    <span className={isLast ? 'text-text-secondary' : 'text-text-tertiary'}>
                      {crumb.label}
                    </span>
                  )}
                </li>
              );
            })}
          </ol>
        </nav>
      )}

      {/* Title and actions row */}
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-text-primary">{title}</h1>
        {children && (
          <div className="flex items-center gap-2">{children}</div>
        )}
      </div>
    </div>
  );
}
