import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { PageHeader } from './PageHeader';

describe('PageHeader', () => {
  it('renders the title', () => {
    render(<PageHeader title="Sessions" />);

    expect(screen.getByText('Sessions')).toBeInTheDocument();
    expect(screen.getByText('Sessions').tagName).toBe('H1');
  });

  it('renders breadcrumbs when provided', () => {
    render(
      <PageHeader
        title="Dashboard"
        breadcrumbs={[
          { label: 'Home', href: '/' },
          { label: 'Analytics', href: '/analytics' },
          { label: 'Reports' },
        ]}
      />,
    );

    expect(screen.getByText('Home')).toBeInTheDocument();
    expect(screen.getByText('Reports')).toBeInTheDocument();
    expect(screen.getByText('Analytics')).toBeInTheDocument();
    expect(screen.getByText('Dashboard')).toBeInTheDocument();
  });

  it('renders action buttons in the action area', () => {
    render(
      <PageHeader title="Sessions">
        <button>Create Session</button>
        <button>Filter</button>
      </PageHeader>,
    );

    expect(screen.getByText('Create Session')).toBeInTheDocument();
    expect(screen.getByText('Filter')).toBeInTheDocument();
  });

  it('does not render breadcrumbs section when not provided', () => {
    render(<PageHeader title="Sessions" />);

    // The breadcrumb nav should not be present
    expect(screen.queryByRole('navigation')).not.toBeInTheDocument();
  });
});
