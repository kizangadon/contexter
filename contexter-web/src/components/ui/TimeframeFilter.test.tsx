import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { TimeframeFilter } from './TimeframeFilter';

describe('TimeframeFilter', () => {
  it('renders with the default value', () => {
    render(<TimeframeFilter value="30d" onChange={() => {}} />);
    const select = screen.getByRole('combobox');
    expect(select).toBeInTheDocument();
    expect((select as HTMLSelectElement).value).toBe('30d');
  });

  it('renders all preset options', () => {
    render(<TimeframeFilter value="7d" onChange={() => {}} />);
    expect(screen.getByText('Last 7 days')).toBeInTheDocument();
    expect(screen.getByText('Last 30 days')).toBeInTheDocument();
    expect(screen.getByText('Last 90 days')).toBeInTheDocument();
    expect(screen.getByText('All time')).toBeInTheDocument();
  });

  it('selecting a preset calls onChange with the new value', async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();
    render(<TimeframeFilter value="7d" onChange={onChange} />);

    const select = screen.getByRole('combobox');
    await user.selectOptions(select, '90d');

    expect(onChange).toHaveBeenCalledWith('90d');
  });

  it('renders custom option with "Custom" label', () => {
    render(<TimeframeFilter value="7d" onChange={() => {}} />);
    expect(screen.getByText('Custom')).toBeInTheDocument();
  });

  it('shows date inputs when value is "custom"', () => {
    const { container } = render(<TimeframeFilter value="custom" onChange={() => {}} />);
    const dateRangeInputs = container.querySelectorAll('input[type="date"]');
    expect(dateRangeInputs.length).toBe(2);
  });

  it('does not show date inputs when value is a preset', () => {
    const { container } = render(<TimeframeFilter value="30d" onChange={() => {}} />);
    const dateRangeInputs = container.querySelectorAll('input[type="date"]');
    expect(dateRangeInputs.length).toBe(0);
  });

  it('date inputs have the correct aria labels', () => {
    render(<TimeframeFilter value="custom" onChange={() => {}} />);
    expect(screen.getByLabelText('Start date')).toBeInTheDocument();
    expect(screen.getByLabelText('End date')).toBeInTheDocument();
  });

  it('applies custom className', () => {
    const { container } = render(
      <TimeframeFilter value="30d" onChange={() => {}} className="custom-filter" />,
    );
    const outerDiv = container.firstChild as HTMLElement;
    expect(outerDiv.className).toContain('custom-filter');
  });
});
