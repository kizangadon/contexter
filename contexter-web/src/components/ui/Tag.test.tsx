import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { Tag } from './Tag';

describe('Tag', () => {
  it('renders the label text', () => {
    render(<Tag label="memory" />);
    expect(screen.getByText('memory')).toBeInTheDocument();
  });

  it('renders with a color variant', () => {
    const { container } = render(<Tag label="success" color="success" />);
    const tag = container.firstChild as HTMLElement;
    expect(tag.className).toContain('bg-success');
  });

  it('renders with default styling when no color is provided', () => {
    const { container } = render(<Tag label="default" />);
    const tag = container.firstChild as HTMLElement;
    expect(tag.className).toContain('bg-bg-tertiary');
  });

  it('shows remove button when onRemove is provided', () => {
    const onRemove = vi.fn();
    render(<Tag label="removable" onRemove={onRemove} />);
    expect(screen.getByRole('button')).toBeInTheDocument();
  });

  it('does not show remove button when onRemove is not provided', () => {
    render(<Tag label="no-remove" />);
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });

  it('fires onRemove when remove button is clicked', async () => {
    const onRemove = vi.fn();
    const user = userEvent.setup();
    render(<Tag label="clickable" onRemove={onRemove} />);
    await user.click(screen.getByRole('button'));
    expect(onRemove).toHaveBeenCalledTimes(1);
  });

  it('truncates text longer than 50 characters', () => {
    const longText = 'a'.repeat(60);
    const { container } = render(<Tag label={longText} />);
    const tag = container.firstChild as HTMLElement;
    expect(tag.className).toContain('truncate');
  });

  it('applies custom className', () => {
    const { container } = render(<Tag label="test" className="custom-class" />);
    const tag = container.firstChild as HTMLElement;
    expect(tag.className).toContain('custom-class');
  });
});
