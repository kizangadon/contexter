import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { LoadingSkeleton } from './LoadingSkeleton';

describe('LoadingSkeleton', () => {
  it('renders text skeleton', () => {
    render(<LoadingSkeleton variant="text" />);
    const skeleton = screen.getByTestId('skeleton');
    expect(skeleton).toBeInTheDocument();
  });

  it('renders card skeleton', () => {
    render(<LoadingSkeleton variant="card" />);
    const skeleton = screen.getByTestId('skeleton');
    expect(skeleton).toBeInTheDocument();
  });

  it('renders table-row skeleton', () => {
    render(<LoadingSkeleton variant="table-row" />);
    const skeleton = screen.getByTestId('skeleton');
    expect(skeleton).toBeInTheDocument();
  });

  it('renders avatar skeleton', () => {
    render(<LoadingSkeleton variant="avatar" />);
    const skeleton = screen.getByTestId('skeleton');
    expect(skeleton).toBeInTheDocument();
  });

  it('renders N items for count prop', () => {
    render(<LoadingSkeleton variant="text" count={3} />);
    const skeletons = screen.getAllByTestId('skeleton');
    expect(skeletons).toHaveLength(3);
  });

  it('renders single item by default', () => {
    render(<LoadingSkeleton variant="text" />);
    const skeletons = screen.getAllByTestId('skeleton');
    expect(skeletons).toHaveLength(1);
  });

  it('applies custom className', () => {
    render(
      <LoadingSkeleton variant="text" className="custom-skeleton-class" />,
    );
    const skeleton = screen.getByTestId('skeleton');
    expect(skeleton.className).toContain('custom-skeleton-class');
  });
});
