import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { MemoryRouter } from 'react-router';
import { TopBar } from './TopBar';

function Wrapper({ children }: { children: React.ReactNode }) {
  return <MemoryRouter>{children}</MemoryRouter>;
}

describe('TopBar', () => {
  it('renders breadcrumbs', () => {
    render(
      <TopBar
        breadcrumbs={[
          { label: 'Home' },
          { label: 'Sessions' },
        ]}
      />,
      { wrapper: Wrapper },
    );

    expect(screen.getByText('Home')).toBeInTheDocument();
    expect(screen.getByText('Sessions')).toBeInTheDocument();
  });

  it('renders breadcrumb with links', () => {
    render(
      <TopBar
        breadcrumbs={[
          { label: 'Home', href: '/' },
          { label: 'Sessions', href: '/sessions' },
          { label: 'Current' },
        ]}
      />,
      { wrapper: Wrapper },
    );

    const homeLink = screen.getByText('Home').closest('a');
    expect(homeLink).toHaveAttribute('href', '/');

    const sessionsLink = screen.getByText('Sessions').closest('a');
    expect(sessionsLink).toHaveAttribute('href', '/sessions');

    // Last breadcrumb (no href) should not be a link
    const current = screen.getByText('Current');
    expect(current.tagName).not.toBe('A');
  });

  it('renders search trigger button', () => {
    render(<TopBar breadcrumbs={[{ label: 'Home' }]} />, { wrapper: Wrapper });

    const searchButton = screen.getByLabelText('Search (⌘K)');
    expect(searchButton).toBeInTheDocument();
  });

  it('renders notification bell with badge count', () => {
    render(
      <TopBar
        breadcrumbs={[{ label: 'Home' }]}
        notificationCount={5}
      />,
      { wrapper: Wrapper },
    );

    const notificationButton = screen.getByLabelText('Notifications');
    expect(notificationButton).toBeInTheDocument();
    expect(screen.getByText('5')).toBeInTheDocument();
  });

  it('renders without badge when count is 0', () => {
    render(
      <TopBar
        breadcrumbs={[{ label: 'Home' }]}
        notificationCount={0}
      />,
      { wrapper: Wrapper },
    );

    const notificationButton = screen.getByLabelText('Notifications');
    expect(notificationButton).toBeInTheDocument();
    expect(screen.queryByText('0')).not.toBeInTheDocument();
  });

  it('renders user avatar with initials', () => {
    render(<TopBar breadcrumbs={[{ label: 'Home' }]} />, { wrapper: Wrapper });

    // Should show a circle with initials
    const avatar = screen.getByLabelText('User menu');
    expect(avatar).toBeInTheDocument();
  });
});
