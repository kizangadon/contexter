import { Suspense } from 'react';
import { Outlet, useLocation } from 'react-router';
import {
  LayoutDashboard,
  MessageSquare,
  Database,
  Bot,
  Puzzle,
  BarChart3,
  Gauge,
  Search,
  Settings,
  Bell,
  MessageCircle,
  LogIn,
  Play,
  Download,
  Share2,
  FileText,
} from 'lucide-react';
import { AppShell } from './AppShell';
import type { NavItem } from './SidebarNav';
import { pathToBreadcrumbs } from '@/components/ui/Breadcrumb';

/* ─── Navigation item definitions ─────────────────────────────── */

// oxlint-disable-next-line react/only-export-components — NAV_ITEMS is a data constant, not a component; co-located intentionally
export const NAV_ITEMS: NavItem[] = [
  { id: 'dashboard', label: 'Dashboard', icon: LayoutDashboard, href: '/dashboard', section: 'Core' },
  { id: 'sessions', label: 'Sessions', icon: MessageSquare, href: '/sessions', section: 'Core' },
  { id: 'memories', label: 'Memories', icon: Database, href: '/memories', section: 'Core' },
  { id: 'agents', label: 'Agents', icon: Bot, href: '/agents', section: 'Core' },
  { id: 'skills', label: 'Skills', icon: Puzzle, href: '/skills', section: 'Core' },
  { id: 'analytics', label: 'Analytics', icon: BarChart3, href: '/analytics', section: 'Core' },

  { id: 'efficiency', label: 'Efficiency', icon: Gauge, href: '/efficiency', section: 'Intelligence' },
  { id: 'search', label: 'Search', icon: Search, href: '/search', section: 'Intelligence' },
  { id: 'correlation', label: 'Correlation', icon: Share2, href: '/correlation', section: 'Intelligence' },
  { id: 'playground', label: 'Playground', icon: Play, href: '/playground', section: 'Intelligence' },

  { id: 'audit', label: 'Audit', icon: FileText, href: '/audit', section: 'Compliance' },
  { id: 'exports', label: 'Exports', icon: Download, href: '/exports', section: 'Compliance' },

  { id: 'notifications', label: 'Notifications', icon: Bell, href: '/notifications', section: 'System' },
  { id: 'feedback', label: 'Feedback', icon: MessageCircle, href: '/feedback', section: 'System' },
  { id: 'onboarding', label: 'Onboarding', icon: LogIn, href: '/onboarding', section: 'System' },
  { id: 'settings', label: 'Settings', icon: Settings, href: '/settings', section: 'System' },
];

/* ─── Root layout component ──────────────────────────────────── */

export function RootLayout() {
  const location = useLocation();

  // Determine active nav item from the first path segment
  const pathSegments = location.pathname.split('/').filter(Boolean);
  const rootSegment = pathSegments[0] || 'dashboard';
  const activeItemId = rootSegment;

  const breadcrumbs = pathToBreadcrumbs(location.pathname);

  return (
    <AppShell
      navItems={NAV_ITEMS}
      breadcrumbs={breadcrumbs}
      activeItemId={activeItemId}
    >
      <Suspense
        fallback={
          <div className="flex items-center justify-center min-h-[400px]">
            <div className="animate-spin rounded-full h-8 w-8 border-t-2 border-[#7C5CFC]" />
          </div>
        }
      >
        <Outlet />
      </Suspense>
    </AppShell>
  );
}
