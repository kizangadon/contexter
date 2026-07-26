import { render, screen } from '@testing-library/react';
import { Inbox } from 'lucide-react';
import { describe, expect, it } from 'vitest';
import { EmptyState } from './EmptyState';

describe('EmptyState', () => {
  it('renders icon, title, and message', () => {
    const { container } = render(
      <EmptyState
        icon={Inbox}
        title="No messages"
        message="You have no messages yet"
      />,
    );

    expect(screen.getByText('No messages')).toBeInTheDocument();
    expect(screen.getByText('You have no messages yet')).toBeInTheDocument();
    // Icon renders as SVG
    const svg = container.querySelector('svg');
    expect(svg).toBeInTheDocument();
  });

  it('renders CTA button when action is provided', () => {
    render(
      <EmptyState
        icon={Inbox}
        title="No data"
        message="Nothing here yet"
        action={<button>Create Item</button>}
      />,
    );

    expect(
      screen.getByRole('button', { name: 'Create Item' }),
    ).toBeInTheDocument();
  });

  it('does not render CTA when action is not provided', () => {
    render(
      <EmptyState icon={Inbox} title="No data" message="Nothing here" />,
    );

    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });

  it('renders without icon', () => {
    render(<EmptyState title="No data" message="No data available" />);

    expect(screen.getByText('No data')).toBeInTheDocument();
    expect(screen.getByText('No data available')).toBeInTheDocument();
  });

  it('renders with centered layout', () => {
    const { container } = render(
      <EmptyState title="Empty" message="Nothing to see" />,
    );

    const wrapper = container.firstElementChild;
    expect(wrapper?.className).toContain('flex');
    expect(wrapper?.className).toContain('items-center');
    expect(wrapper?.className).toContain('justify-center');
  });
});
