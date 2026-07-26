import { type ReactNode } from 'react';
import { Outlet } from 'react-router';
import { SidebarProvider, useSidebar } from './SidebarContext';
import { SidebarNav, type NavItem } from './SidebarNav';
import { TopBar, type Breadcrumb } from './TopBar';

interface AppShellProps {
  /** Navigation items for the sidebar */
  navItems: NavItem[];
  /** Breadcrumb trail for the top bar */
  breadcrumbs: Breadcrumb[];
  /** Optional: render children instead of Outlet */
  children?: ReactNode;
  /** Active navigation item ID */
  activeItemId?: string;
  /** Number of unread notifications */
  notificationCount?: number;
}

function ShellLayout({
  navItems,
  breadcrumbs,
  children,
  activeItemId,
  notificationCount,
}: AppShellProps) {
  const { isCollapsed } = useSidebar();

  return (
    <div
      data-testid="app-shell"
      className="grid h-screen overflow-hidden"
      style={{
        gridTemplateColumns: `${isCollapsed ? '60px' : '240px'} 1fr`,
        gridTemplateRows: '56px 1fr',
        transition: 'grid-template-columns 300ms',
      }}
    >
      {/* Sidebar — fixed to left, spans full height */}
      <div className="row-span-2 overflow-hidden">
        <SidebarNav items={navItems} activeItemId={activeItemId} />
      </div>

      {/* Top bar — top right */}
      <TopBar breadcrumbs={breadcrumbs} notificationCount={notificationCount} />

      {/* Content area — bottom right */}
      <main className="mx-auto max-w-[1440px] overflow-auto bg-bg-secondary p-6">
        {children ?? <Outlet />}
      </main>
    </div>
  );
}

export function AppShell(props: AppShellProps) {
  return (
    <SidebarProvider>
      <ShellLayout {...props} />
    </SidebarProvider>
  );
}
