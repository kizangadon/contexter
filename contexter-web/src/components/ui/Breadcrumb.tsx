/**
 * Breadcrumb component and path-to-breadcrumb utility.
 *
 * Converts a URL pathname into a breadcrumb trail and renders it
 * as a semantic `<nav>` element with an ordered list.
 */

/* ── Types ─────────────────────────────────────────────────── */

export interface BreadcrumbItem {
  /** Display label for the breadcrumb */
  label: string;
  /** Optional href — last breadcrumb typically has no href */
  href?: string;
}

/* ── Utility ───────────────────────────────────────────────── */

/**
 * Convert a URL pathname into an array of BreadcrumbItem objects.
 *
 * Example: `/memories/abc-123` → `[{label:'Home',href:'/'}, {label:'Memories',href:'/memories'}, {label:'Abc-123'}]`
 */
// oxlint-disable-next-line react/only-export-components — utility function co-located with its companion component
export function pathToBreadcrumbs(pathname: string): BreadcrumbItem[] {
  const segments = pathname.split('/').filter(Boolean);
  if (segments.length === 0) return [{ label: 'Dashboard', href: '/dashboard' }];

  const crumbs: BreadcrumbItem[] = [{ label: 'Home', href: '/' }];

  for (let i = 0; i < segments.length; i++) {
    const segment = segments[i]!;
    const href = '/' + segments.slice(0, i + 1).join('/');
    const isLast = i === segments.length - 1;
    const label = segment.charAt(0).toUpperCase() + segment.slice(1).replace(/-/g, ' ');

    if (isLast) {
      crumbs.push({ label });
    } else {
      crumbs.push({ label, href });
    }
  }

  return crumbs;
}

/* ── Component ─────────────────────────────────────────────── */

interface BreadcrumbProps {
  /** Breadcrumb trail items */
  items: BreadcrumbItem[];
}

export function Breadcrumb({ items }: BreadcrumbProps) {
  if (items.length === 0) return null;

  return (
    <nav aria-label="Breadcrumb">
      <ol className="flex items-center gap-1.5 text-sm">
        {items.map((crumb, index) => {
          const isLast = index === items.length - 1;

          return (
            <li
              key={`${crumb.label}-${index}`}
              className="flex items-center gap-1.5"
            >
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
                <span
                  className={
                    isLast
                      ? 'font-medium text-text-primary'
                      : 'text-text-secondary'
                  }
                >
                  {crumb.label}
                </span>
              )}
            </li>
          );
        })}
      </ol>
    </nav>
  );
}
