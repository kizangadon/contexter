import { ChevronLeft, ChevronRight, ChevronDown, ChevronRight as ChevronRightIcon } from 'lucide-react';
import { type LucideIcon } from 'lucide-react';
import { useState, useMemo } from 'react';
import { Link, useLocation } from 'react-router';
import { useSidebar } from './SidebarContext';

export interface NavItem {
  /** Unique identifier for the navigation item */
  id: string;
  /** Display label */
  label: string;
  /** Lucide icon component */
  icon: LucideIcon;
  /** Route href (optional if item has children) */
  href?: string;
  /** Nested child items */
  children?: NavItem[];
  /** Section grouping label */
  section?: string;
}

interface SidebarNavProps {
  /** Navigation items to render */
  items: NavItem[];
  /** ID of the currently active navigation item */
  activeItemId?: string;
}

function NavItemLink({
  item,
  isCollapsed,
  activeItemId,
  depth = 0,
}: {
  item: NavItem;
  isCollapsed: boolean;
  activeItemId?: string;
  depth?: number;
}) {
  const location = useLocation();
  const Icon = item.icon;
  const isActive = activeItemId === item.id || (item.href !== undefined && location.pathname === item.href);
  const hasChildren = item.children && item.children.length > 0;
  const [expanded, setExpanded] = useState(true);

  const linkContent = (
    <>
      <Icon className="h-5 w-5 shrink-0" aria-hidden="true" />
      {!isCollapsed && (
        <span className="ml-3 truncate text-sm font-medium">{item.label}</span>
      )}
      {!isCollapsed && hasChildren && (
        <span className="ml-auto">
          {expanded ? (
            <ChevronDown className="h-4 w-4 text-text-tertiary" />
          ) : (
            <ChevronRightIcon className="h-4 w-4 text-text-tertiary" />
          )}
        </span>
      )}
    </>
  );

  const activeBorderClass = isActive ? 'border-l-accent' : 'border-l-transparent';
  const hoverClass = 'hover:bg-bg-hover';

  return (
    <li>
      {item.href && !hasChildren ? (
        <Link
          to={item.href!}
          title={isCollapsed ? item.label : undefined}
          className={`flex items-center px-4 py-2.5 text-text-secondary transition-colors duration-150
            border-l-3 ${activeBorderClass} ${hoverClass}
            ${isActive ? 'bg-accent-subtle text-accent' : ''}
            ${depth > 0 ? 'pl-10' : ''}`}
          style={isActive ? { borderLeftWidth: '3px' } : undefined}
        >
          {linkContent}
        </Link>
      ) : (
        <button
          onClick={() => hasChildren && setExpanded(!expanded)}
          title={isCollapsed ? item.label : undefined}
          className={`flex w-full items-center px-4 py-2.5 text-text-secondary transition-colors duration-150
            border-l-3 ${activeBorderClass} ${hoverClass}
            ${isActive ? 'bg-accent-subtle text-accent' : ''}
            ${depth > 0 ? 'pl-10' : ''}`}
        >
          {linkContent}
        </button>
      )}
      {hasChildren && expanded && !isCollapsed && (
        <ul className="mt-0.5 space-y-0.5">
          {item.children!.map((child) => (
            <NavItemLink
              key={child.id}
              item={child}
              isCollapsed={isCollapsed}
              activeItemId={activeItemId}
              depth={depth + 1}
            />
          ))}
        </ul>
      )}
    </li>
  );
}

export function SidebarNav({ items, activeItemId }: SidebarNavProps) {
  const { isCollapsed, toggle } = useSidebar();

  // Group items by section
  const grouped = useMemo(() => {
    const groups: { section: string; items: NavItem[] }[] = [];
    const uncategorized: NavItem[] = [];

    for (const item of items) {
      if (item.section) {
        let group = groups.find((g) => g.section === item.section);
        if (!group) {
          group = { section: item.section, items: [] };
          groups.push(group);
        }
        group.items.push(item);
      } else {
        uncategorized.push(item);
      }
    }

    return { groups, uncategorized };
  }, [items]);

  return (
    <aside
      className={`flex h-full flex-col border-r border-border bg-bg-primary transition-all duration-300 ${
        isCollapsed ? 'w-[60px]' : 'w-[240px]'
      }`}
      aria-label="Main navigation"
    >
      {/* Logo area */}
      <div className="flex h-14 items-center border-b border-border px-4">
        {!isCollapsed && (
          <span className="text-lg font-bold text-text-primary">Contexter</span>
        )}
        {isCollapsed && (
          <span className="mx-auto text-lg font-bold text-text-primary">C</span>
        )}
      </div>

      {/* Navigation items with section grouping */}
      <nav className="flex-1 overflow-y-auto py-2">
        <ul className="space-y-0.5 px-0">
          {/* Uncategorized items first */}
          {grouped.uncategorized.map((item) => (
            <NavItemLink
              key={item.id}
              item={item}
              isCollapsed={isCollapsed}
              activeItemId={activeItemId}
            />
          ))}

          {/* Grouped items with section labels */}
          {grouped.groups.map((group) => {
            return (
              <li key={group.section} className="mt-1">
                {/* Section header */}
                {!isCollapsed && (
                  <div className="px-4 pb-1 pt-3">
                    <span className="text-[10px] font-semibold uppercase tracking-widest text-text-tertiary">
                      {group.section}
                    </span>
                  </div>
                )}
                <ul className="space-y-0.5">
                  {group.items.map((item) => (
                    <NavItemLink
                      key={item.id}
                      item={item}
                      isCollapsed={isCollapsed}
                      activeItemId={activeItemId}
                    />
                  ))}
                </ul>
              </li>
            );
          })}
        </ul>
      </nav>

      {/* Collapse toggle button at bottom */}
      <div className="border-t border-border p-2">
        <button
          onClick={toggle}
          aria-label={isCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
          className="flex w-full items-center justify-center rounded-md px-2 py-2 text-text-secondary transition-colors duration-150 hover:bg-bg-hover hover:text-text-primary"
        >
          {isCollapsed ? (
            <ChevronRight className="h-5 w-5" />
          ) : (
            <ChevronLeft className="h-5 w-5" />
          )}
        </button>
      </div>
    </aside>
  );
}
