import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { FilterBar } from './FilterBar';
import type { FilterDef } from './FilterBar';

const mockFilters: FilterDef[] = [
  {
    key: 'status',
    label: 'Status',
    options: [
      { value: '', label: 'All' },
      { value: 'active', label: 'Active' },
      { value: 'inactive', label: 'Inactive' },
    ],
    value: '',
    onChange: vi.fn(),
  },
  {
    key: 'type',
    label: 'Type',
    options: [
      { value: '', label: 'All Types' },
      { value: 'agent', label: 'Agent' },
      { value: 'user', label: 'User' },
    ],
    value: '',
    onChange: vi.fn(),
  },
];

describe('FilterBar', () => {
  it('renders all filter dropdowns', () => {
    render(<FilterBar filters={mockFilters} />);
    expect(screen.getByLabelText('Status')).toBeInTheDocument();
    expect(screen.getByLabelText('Type')).toBeInTheDocument();
  });

  it('renders filter labels', () => {
    render(<FilterBar filters={mockFilters} />);
    expect(screen.getByText('Status')).toBeInTheDocument();
    expect(screen.getByText('Type')).toBeInTheDocument();
  });

  it('changing a filter fires onChange with key and value', async () => {
    const onStatusChange = vi.fn();
    const filters = [
      { ...mockFilters[0]!, onChange: onStatusChange },
      { ...mockFilters[1]! },
    ];
    const user = userEvent.setup();
    render(<FilterBar filters={filters} />);

    const select = screen.getByLabelText('Status');
    await user.selectOptions(select, 'active');

    expect(onStatusChange).toHaveBeenCalledWith('active');
  });

  it('renders search input when onSearch is provided', () => {
    render(<FilterBar filters={mockFilters} onSearch={() => {}} />);
    expect(screen.getByPlaceholderText('Search…')).toBeInTheDocument();
  });

  it('does not render search input when onSearch is not provided', () => {
    render(<FilterBar filters={mockFilters} />);
    expect(screen.queryByPlaceholderText('Search…')).not.toBeInTheDocument();
  });

  it('search input fires onSearch callback with the full value', async () => {
    const onSearch = vi.fn();
    const user = userEvent.setup();
    render(<FilterBar filters={mockFilters} onSearch={onSearch} />);

    const input = screen.getByPlaceholderText('Search…');
    await user.type(input, 'test');

    expect(onSearch).toHaveBeenLastCalledWith('test');
  });

  it('renders with search placeholder text', () => {
    render(
      <FilterBar filters={mockFilters} onSearch={() => {}} searchPlaceholder="Filter…" />,
    );
    expect(screen.getByPlaceholderText('Filter…')).toBeInTheDocument();
  });

  it('applies custom className', () => {
    const { container } = render(
      <FilterBar filters={mockFilters} className="custom-bar" />,
    );
    const outerDiv = container.firstChild as HTMLElement;
    expect(outerDiv.className).toContain('custom-bar');
  });
});
