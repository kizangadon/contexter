import { useEffect } from 'react';
import { useNavigate } from 'react-router';
import { Bell, Search } from 'lucide-react';
import { Breadcrumb, type BreadcrumbItem } from '@/components/ui/Breadcrumb';

export type { BreadcrumbItem as Breadcrumb };

interface TopBarProps {
  /** Breadcrumb trail items */
  breadcrumbs: BreadcrumbItem[];
  /** Number of unread notifications (0 hides badge) */
  notificationCount?: number;
}

export function TopBar({ breadcrumbs, notificationCount = 0 }: TopBarProps) {
  const navigate = useNavigate();

  // ⌘K / Ctrl+K keyboard shortcut to navigate to search
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        navigate('/search');
      }
    };
    document.addEventListener('keydown', handler);
    return () => document.removeEventListener('keydown', handler);
  }, [navigate]);

  return (
    <header
      className="flex h-14 items-center justify-between border-b border-border bg-bg-primary px-4"
      role="banner"
    >
      <Breadcrumb items={breadcrumbs} />

      {/* Right section: search, notifications, avatar */}
      <div className="flex items-center gap-2">
        {/* Search trigger */}
        <button
          aria-label="Search (⌘K)"
          title="Search (⌘K)"
          className="flex h-9 w-9 items-center justify-center rounded-md text-text-secondary transition-colors duration-150 hover:bg-bg-hover hover:text-text-primary"
        >
          <Search className="h-5 w-5" />
        </button>

        {/* Notifications */}
        <button
          aria-label="Notifications"
          className="relative flex h-9 w-9 items-center justify-center rounded-md text-text-secondary transition-colors duration-150 hover:bg-bg-hover hover:text-text-primary"
        >
          <Bell className="h-5 w-5" />
          {notificationCount > 0 && (
            <span className="absolute -right-0.5 -top-0.5 flex h-4 min-w-[16px] items-center justify-center rounded-full bg-accent px-1 text-[10px] font-bold leading-none text-text-inverse">
              {notificationCount > 99 ? '99+' : notificationCount}
            </span>
          )}
        </button>

        {/* User avatar */}
        <button
          aria-label="User menu"
          className="ml-2 flex h-8 w-8 items-center justify-center rounded-full bg-accent text-sm font-bold text-text-inverse transition-opacity duration-150 hover:opacity-90"
        >
          CN
        </button>
      </div>
    </header>
  );
}
