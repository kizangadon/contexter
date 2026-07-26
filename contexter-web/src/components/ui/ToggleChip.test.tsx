import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { ToggleChip } from './ToggleChip';

describe('ToggleChip', () => {
  it('renders with label text', () => {
    render(<ToggleChip onClick={vi.fn()}>Label</ToggleChip>);
    expect(screen.getByRole('button', { name: 'Label' })).toBeInTheDocument();
  });

  it('renders as inactive by default', () => {
    render(<ToggleChip onClick={vi.fn()}>Inactive</ToggleChip>);
    const chip = screen.getByRole('button');
    expect(chip.className).toContain('bg-bg-tertiary');
    expect(chip.className).toContain('text-text-secondary');
  });

  it('renders as active when active prop is true', () => {
    render(
      <ToggleChip active onClick={vi.fn()}>
        Active
      </ToggleChip>,
    );
    const chip = screen.getByRole('button');
    expect(chip.className).toContain('bg-accent');
    expect(chip.className).toContain('text-text-inverse');
  });

  it('fires onClick when clicked', async () => {
    const user = userEvent.setup();
    const handleClick = vi.fn();
    render(<ToggleChip onClick={handleClick}>Clickable</ToggleChip>);

    await user.click(screen.getByRole('button'));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('applies custom className', () => {
    render(
      <ToggleChip onClick={vi.fn()} className="my-custom-class">
        Styled
      </ToggleChip>,
    );
    const chip = screen.getByRole('button');
    expect(chip.className).toContain('my-custom-class');
  });
});
