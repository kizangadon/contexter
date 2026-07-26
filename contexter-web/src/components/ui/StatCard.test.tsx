import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { StatCard } from './StatCard';
import type { Trend } from './StatCard';

describe('StatCard', () => {
  it('renders the value and label', () => {
    render(<StatCard value="1,234" label="Total Sessions" />);
    expect(screen.getByText('1,234')).toBeInTheDocument();
    expect(screen.getByText('Total Sessions')).toBeInTheDocument();
  });

  it('renders a numeric value', () => {
    render(<StatCard value={42} label="Active Agents" />);
    expect(screen.getByText('42')).toBeInTheDocument();
  });

  it('renders an up trend in green', () => {
    const trend: Trend = { direction: 'up', percentage: 12.5 };
    render(<StatCard value="100" label="Test" trend={trend} />);
    expect(screen.getByText('12.5%')).toBeInTheDocument();
    expect(screen.getByText('12.5%').className).toContain('text-success');
  });

  it('renders a down trend in red', () => {
    const trend: Trend = { direction: 'down', percentage: 5.2 };
    render(<StatCard value="100" label="Test" trend={trend} />);
    expect(screen.getByText('5.2%')).toBeInTheDocument();
    const el = screen.getByText('5.2%');
    expect(el.className).toContain('text-error');
  });

  it('renders a neutral trend in gray', () => {
    const trend: Trend = { direction: 'neutral', percentage: 0 };
    render(<StatCard value="100" label="Test" trend={trend} />);
    expect(screen.getByText('0%')).toBeInTheDocument();
    const el = screen.getByText('0%');
    expect(el.className).toContain('text-text-tertiary');
  });

  it('shows loading skeleton when loading is true', () => {
    const { container } = render(<StatCard value="100" label="Test" loading />);
    const skeletons = container.querySelectorAll('[data-testid="skeleton"]');
    expect(skeletons.length).toBeGreaterThan(0);
  });

  it('does not show loading skeleton when loading is false', () => {
    const { container } = render(<StatCard value="100" label="Test" />);
    const skeletons = container.querySelectorAll('[data-testid="skeleton"]');
    expect(skeletons.length).toBe(0);
  });

  it('applies custom className', () => {
    const { container } = render(
      <StatCard value="100" label="Test" className="custom-class" />,
    );
    const card = container.firstChild as HTMLElement;
    expect(card.className).toContain('custom-class');
  });
});
