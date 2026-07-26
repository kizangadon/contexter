import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { Badge } from './Badge';

describe('Badge', () => {
  it('renders label text', () => {
    render(<Badge>Active</Badge>);
    expect(screen.getByText('Active')).toBeInTheDocument();
  });

  it.each(['success', 'warning', 'error', 'info', 'pending', 'offline'] as const)(
    'applies correct styling for %s variant',
    (variant) => {
      render(<Badge variant={variant}>{variant}</Badge>);
      const badge = screen.getByText(variant);
      expect(badge.className).toContain(variant);
    },
  );

  it('renders sm size', () => {
    render(<Badge size="sm">Small</Badge>);
    const badge = screen.getByText('Small');
    expect(badge.className).toContain('text-xs');
    expect(badge.className).toContain('px-1');
  });

  it('renders md size by default', () => {
    render(<Badge>Medium</Badge>);
    const badge = screen.getByText('Medium');
    expect(badge.className).toContain('text-sm');
    expect(badge.className).toContain('px-2');
  });

  it('renders dot indicator when dot prop is true', () => {
    const { container } = render(
      <Badge dot variant="success">
        With Dot
      </Badge>,
    );
    // The dot should be a span element inside the badge
    const dot = container.querySelector('span');
    expect(dot).toBeInTheDocument();
    expect(dot?.className).toContain('bg-success');
  });

  it('does not render dot when dot prop is false', () => {
    const { container } = render(<Badge>No Dot</Badge>);
    // The only child is the text node — no span dot
    const badgeEl = container.firstElementChild;
    expect(badgeEl?.querySelector('span')).toBeNull();
  });
});
