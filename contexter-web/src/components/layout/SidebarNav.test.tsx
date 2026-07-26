import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {
  BarChart3,
  Database,
  FileText,
  Gauge,
  LayoutDashboard,
  Settings,
  type LucideIcon,
} from 'lucide-react';
import { MemoryRouter } from 'react-router';
import { describe, expect, it } from 'vitest';
import { SidebarProvider } from './SidebarContext';
import { SidebarNav, type NavItem } from './SidebarNav';

const testItems: NavItem[] = [
  { id: 'dashboard', label: 'Dashboard', icon: LayoutDashboard as LucideIcon, href: '/dashboard' },
  { id: 'sessions', label: 'Sessions', icon: Database as LucideIcon, href: '/sessions' },
  {
    id: 'analytics',
    label: 'Analytics',
    icon: BarChart3 as LucideIcon,
    children: [
      { id: 'efficiency', label: 'Efficiency', icon: Gauge as LucideIcon, href: '/analytics/efficiency' },
      { id: 'reports', label: 'Reports', icon: FileText as LucideIcon, href: '/analytics/reports' },
    ],
  },
  { id: 'settings', label: 'Settings', icon: Settings as LucideIcon, href: '/settings' },
];

function renderSidebarNav(activeItemId?: string) {
  return render(
    <MemoryRouter>
      <SidebarProvider>
        <SidebarNav items={testItems} activeItemId={activeItemId} />
      </SidebarProvider>
    </MemoryRouter>,
  );
}

describe('SidebarNav', () => {
  it('renders all navigation items', () => {
    renderSidebarNav();
    expect(screen.getByText('Dashboard')).toBeInTheDocument();
    expect(screen.getByText('Sessions')).toBeInTheDocument();
    expect(screen.getByText('Analytics')).toBeInTheDocument();
    expect(screen.getByText('Settings')).toBeInTheDocument();
  });

  it('shows labels in expanded state', () => {
    renderSidebarNav();
    expect(screen.getByText('Dashboard')).toBeVisible();
    expect(screen.getByText('Sessions')).toBeVisible();
    expect(screen.getByText('Settings')).toBeVisible();
  });

  it('hides labels in collapsed state', async () => {
    const user = userEvent.setup();
    renderSidebarNav();

    const collapseButton = screen.getByLabelText('Collapse sidebar');
    await user.click(collapseButton);

    expect(screen.queryByText('Dashboard')).not.toBeInTheDocument();
    expect(screen.queryByText('Sessions')).not.toBeInTheDocument();
    expect(screen.queryByText('Settings')).not.toBeInTheDocument();
  });

  it('highlights active item with accent border', () => {
    renderSidebarNav('sessions');

    const sessionsItem = screen.getByText('Sessions').closest('a');
    expect(sessionsItem).toHaveClass('border-l-accent');

    const dashboardItem = screen.getByText('Dashboard').closest('a');
    expect(dashboardItem).not.toHaveClass('border-l-accent');
  });

  it('toggles collapse state when toggle button is clicked', async () => {
    const user = userEvent.setup();
    renderSidebarNav();

    const collapseButton = screen.getByLabelText('Collapse sidebar');
    expect(collapseButton).toBeInTheDocument();

    await user.click(collapseButton);

    const expandButton = screen.getByLabelText('Expand sidebar');
    expect(expandButton).toBeInTheDocument();

    await user.click(expandButton);
    expect(screen.getByLabelText('Collapse sidebar')).toBeInTheDocument();
  });

  it('shows tooltip with label on collapsed nav items', async () => {
    const user = userEvent.setup();
    renderSidebarNav();

    const collapseButton = screen.getByLabelText('Collapse sidebar');
    await user.click(collapseButton);

    const dashboardLink = screen.getByTitle('Dashboard');
    expect(dashboardLink).toBeInTheDocument();
    expect(dashboardLink.tagName).toBe('A');

    const sessionsLink = screen.getByTitle('Sessions');
    expect(sessionsLink).toBeInTheDocument();
    expect(sessionsLink.tagName).toBe('A');
  });

  it('renders collapse button at the bottom of the sidebar', () => {
    renderSidebarNav();

    const collapseButton = screen.getByLabelText('Collapse sidebar');
    expect(collapseButton).toBeInTheDocument();
  });

  it('renders sub-items for items with children', () => {
    renderSidebarNav();

    expect(screen.getByText('Analytics')).toBeInTheDocument();
    expect(screen.getByText('Efficiency')).toBeInTheDocument();
    expect(screen.getByText('Reports')).toBeInTheDocument();
  });

  it('applies hover background class on nav items', () => {
    renderSidebarNav();

    const dashboardLink = screen.getByText('Dashboard').closest('a');
    expect(dashboardLink).toHaveClass('hover:bg-bg-hover');
  });
});
