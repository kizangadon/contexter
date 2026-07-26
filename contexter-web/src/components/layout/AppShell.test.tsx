import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Database, LayoutDashboard, Settings, type LucideIcon } from 'lucide-react';
import { describe, expect, it, vi } from 'vitest';
import { AppShell } from './AppShell';

// Mock react-router — provide stubs for components used by SidebarNav
vi.mock('react-router', () => ({
  Link: ({ to, children, ...props }: Record<string, unknown>) => (
    <a href={String(to)} {...props}>{children as React.ReactNode}</a>
  ),
  useLocation: () => ({ pathname: '/', search: '', hash: '', state: null, key: 'default' }),
  MemoryRouter: ({ children }: { children: React.ReactNode }) => <>{children}</>,
  Outlet: () => <div data-testid="outlet">Outlet content</div>,
  useNavigate: () => vi.fn(),
  useParams: () => ({}),
}));

const testNavItems = [
  { id: 'dashboard', label: 'Dashboard', icon: LayoutDashboard as LucideIcon, href: '/dashboard' },
  { id: 'sessions', label: 'Sessions', icon: Database as LucideIcon, href: '/sessions' },
  { id: 'settings', label: 'Settings', icon: Settings as LucideIcon, href: '/settings' },
];

describe('AppShell', () => {
  it('renders SidebarNav with navigation items', () => {
    render(
      <AppShell navItems={testNavItems} breadcrumbs={[{ label: 'Home' }]}>
        <div>Content</div>
      </AppShell>,
    );

    expect(screen.getByText('Dashboard')).toBeInTheDocument();
    expect(screen.getByText('Sessions')).toBeInTheDocument();
    expect(screen.getByText('Settings')).toBeInTheDocument();
  });

  it('renders TopBar with breadcrumbs', () => {
    render(
      <AppShell
        navItems={testNavItems}
        breadcrumbs={[{ label: 'Home' }, { label: 'Page Title' }]}
      >
        <div>Content</div>
      </AppShell>,
    );

    expect(screen.getByText('Home')).toBeInTheDocument();
    expect(screen.getByText('Page Title')).toBeInTheDocument();
  });

  it('renders children content', () => {
    render(
      <AppShell navItems={testNavItems} breadcrumbs={[{ label: 'Home' }]}>
        <div>Page content here</div>
      </AppShell>,
    );

    expect(screen.getByText('Page content here')).toBeInTheDocument();
  });

  it('renders Outlet when provided', () => {
    render(
      <AppShell
        navItems={testNavItems}
        breadcrumbs={[{ label: 'Home' }]}
      />,
    );

    expect(screen.getByTestId('outlet')).toBeInTheDocument();
  });

  it('toggles sidebar collapse via context', async () => {
    const user = userEvent.setup();
    render(
      <AppShell navItems={testNavItems} breadcrumbs={[{ label: 'Home' }]}>
        <div>Content</div>
      </AppShell>,
    );

    // Initially expanded — sidebar should be visible in full width
    const collapseButton = screen.getByLabelText('Collapse sidebar');
    expect(collapseButton).toBeInTheDocument();

    // Click to collapse
    await user.click(collapseButton);
    expect(screen.getByLabelText('Expand sidebar')).toBeInTheDocument();

    // Check that the sidebar aside element has collapsed width class
    const sidebar = screen.getByLabelText('Main navigation');
    expect(sidebar.className).toContain('w-[60px]');
  });

  it('uses CSS Grid layout with sidebar transition', () => {
    render(
      <AppShell navItems={testNavItems} breadcrumbs={[{ label: 'Home' }]}>
        <div>Content</div>
      </AppShell>,
    );

    // The shell container should have grid layout classes
    const shellContainer = screen.getByTestId('app-shell');
    expect(shellContainer.className).toContain('grid');
  });

  it('renders active nav item based on prop', () => {
    render(
      <AppShell navItems={testNavItems} breadcrumbs={[{ label: 'Home' }]} activeItemId="sessions">
        <div>Content</div>
      </AppShell>,
    );

    const sessionsItem = screen.getByText('Sessions').closest('a');
    expect(sessionsItem).toHaveClass('border-l-accent');
  });
});
