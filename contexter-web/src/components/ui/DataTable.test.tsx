import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { DataTable } from './DataTable';
import type { Column } from './DataTable';
import { Inbox } from 'lucide-react';

interface TestItem {
  id: string;
  name: string;
  status: string;
}

const columns: Column<TestItem>[] = [
  { key: 'name', header: 'Name', sortable: true, render: (item) => item.name },
  { key: 'status', header: 'Status', sortable: true, render: (item) => item.status },
];

const data: TestItem[] = [
  { id: '1', name: 'Alpha', status: 'active' },
  { id: '2', name: 'Beta', status: 'inactive' },
  { id: '3', name: 'Gamma', status: 'active' },
];

describe('DataTable', () => {
  it('renders column headers', () => {
    render(<DataTable columns={columns} data={data} />);
    expect(screen.getByText('Name')).toBeInTheDocument();
    expect(screen.getByText('Status')).toBeInTheDocument();
  });

  it('renders data rows', () => {
    render(<DataTable columns={columns} data={data} />);
    expect(screen.getByText('Alpha')).toBeInTheDocument();
    expect(screen.getByText('Beta')).toBeInTheDocument();
    expect(screen.getByText('Gamma')).toBeInTheDocument();
  });

  it('calls onSort when a sortable header is clicked', async () => {
    const onSort = vi.fn();
    const user = userEvent.setup();
    render(<DataTable columns={columns} data={data} onSort={onSort} sortable />);

    await user.click(screen.getByText('Name'));

    expect(onSort).toHaveBeenCalledWith('name', 'asc');
  });

  it('toggles sort direction on second click', async () => {
    const onSort = vi.fn();
    const user = userEvent.setup();
    render(<DataTable columns={columns} data={data} onSort={onSort} sortable />);

    // First click: asc
    await user.click(screen.getByText('Name'));
    expect(onSort).toHaveBeenNthCalledWith(1, 'name', 'asc');

    // Second click: desc
    await user.click(screen.getByText('Name'));
    expect(onSort).toHaveBeenNthCalledWith(2, 'name', 'desc');
  });

  it('shows sort arrow indicator on click', async () => {
    const user = userEvent.setup();
    const { container } = render(
      <DataTable columns={columns} data={data} onSort={() => {}} sortable />,
    );

    await user.click(screen.getByText('Name'));

    // Sort indicator should be visible (an SVG arrow icon)
    const sortArrows = container.querySelectorAll('svg');
    expect(sortArrows.length).toBeGreaterThan(0);
  });

  it('shows loading skeletons when isLoading is true', () => {
    const { container } = render(
      <DataTable columns={columns} data={[]} isLoading />,
    );
    const skeletons = container.querySelectorAll('[data-testid="skeleton"]');
    expect(skeletons.length).toBeGreaterThan(0);
  });

  it('does not show skeletons when not loading', () => {
    const { container } = render(
      <DataTable columns={columns} data={data} />,
    );
    const skeletons = container.querySelectorAll('[data-testid="skeleton"]');
    expect(skeletons.length).toBe(0);
  });

  it('renders empty state when data is empty and not loading', () => {
    render(
      <DataTable
        columns={columns}
        data={[]}
        emptyState={{
          icon: Inbox,
          title: 'No data',
          message: 'No items to display',
        }}
      />,
    );
    expect(screen.getByText('No data')).toBeInTheDocument();
    expect(screen.getByText('No items to display')).toBeInTheDocument();
  });

  it('calls onRowClick when a row is clicked', async () => {
    const onRowClick = vi.fn();
    const user = userEvent.setup();
    render(
      <DataTable columns={columns} data={data} onRowClick={onRowClick} />,
    );

    await user.click(screen.getByText('Alpha'));

    expect(onRowClick).toHaveBeenCalledWith(
      expect.objectContaining({ id: '1', name: 'Alpha' }),
    );
  });

  it('shows pagination when data exceeds pageSize', () => {
    const manyItems: TestItem[] = Array.from({ length: 25 }, (_, i) => ({
      id: String(i + 1),
      name: `Item ${i + 1}`,
      status: 'active',
    }));

    render(
      <DataTable columns={columns} data={manyItems} pageSize={10} />,
    );

    expect(screen.getByText('Page 1 of 3')).toBeInTheDocument();
    expect(screen.getByText('Previous')).toBeInTheDocument();
    expect(screen.getByText('Next')).toBeInTheDocument();
  });

  it('navigates pages when clicking Next and Previous', async () => {
    const manyItems: TestItem[] = Array.from({ length: 25 }, (_, i) => ({
      id: String(i + 1),
      name: `Item ${i + 1}`,
      status: 'active',
    }));
    const user = userEvent.setup();

    render(
      <DataTable columns={columns} data={manyItems} pageSize={10} />,
    );

    expect(screen.getByText('Page 1 of 3')).toBeInTheDocument();

    // Click Next
    await user.click(screen.getByText('Next'));
    expect(screen.getByText('Page 2 of 3')).toBeInTheDocument();

    // Click Previous
    await user.click(screen.getByText('Previous'));
    expect(screen.getByText('Page 1 of 3')).toBeInTheDocument();
  });

  it('disables Previous on first page, Next on last page', async () => {
    const pageData: TestItem[] = Array.from({ length: 10 }, (_, i) => ({
      id: String(i + 1),
      name: `Item ${i + 1}`,
      status: 'active',
    }));
    const user = userEvent.setup();

    render(
      <DataTable columns={columns} data={pageData} pageSize={5} />,
    );

    // On page 1, Previous should be disabled
    expect(screen.getByText('Previous')).toBeDisabled();
    expect(screen.getByText('Next')).not.toBeDisabled();

    // Go to page 2
    await user.click(screen.getByText('Next'));
    expect(screen.getByText('Page 2 of 2')).toBeInTheDocument();

    // On page 2, Next should be disabled
    expect(screen.getByText('Next')).toBeDisabled();
    expect(screen.getByText('Previous')).not.toBeDisabled();
  });
});
