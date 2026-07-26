import { useState, type ReactNode } from 'react';
import { ArrowUp, ArrowDown, ChevronLeft, ChevronRight } from 'lucide-react';
import type { LucideIcon } from 'lucide-react';
import { LoadingSkeleton } from './LoadingSkeleton';
import { EmptyState } from './EmptyState';

export interface Column<T> {
  /** Unique key for the column */
  key: string;
  /** Display text in the header */
  header: string;
  /** Whether the column is sortable */
  sortable?: boolean;
  /** Render function for each cell */
  render: (item: T) => ReactNode;
  /** Optional fixed width (e.g., '100px', '2fr') */
  width?: string;
}

export interface DataTableProps<T> {
  /** Column definitions */
  columns: Column<T>[];
  /** Row data */
  data: T[];
  /** Show loading skeleton placeholders */
  isLoading?: boolean;
  /** Empty state configuration */
  emptyState?: {
    icon: LucideIcon;
    title: string;
    message: string;
    action?: ReactNode;
  };
  /** Enable sortable headers */
  sortable?: boolean;
  /** Called when a sortable column header is clicked */
  onSort?: (key: string, direction: 'asc' | 'desc') => void;
  /** Called when a row is clicked */
  onRowClick?: (item: T) => void;
  /** Number of rows per page (default: Infinity) */
  pageSize?: number;
  /** Additional CSS class names */
  className?: string;
}

/* ── Styles ────────────────────────────────────────────────── */
const thStyles =
  'border-b border-border px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-text-secondary';

const tdStyles = 'px-4 py-3 text-sm text-text-primary';

export function DataTable<T>({
  columns,
  data,
  isLoading = false,
  emptyState,
  sortable = false,
  onSort,
  onRowClick,
  pageSize = Infinity,
  className = '',
}: DataTableProps<T>) {
  const [sortKey, setSortKey] = useState<string | null>(null);
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('asc');
  const [currentPage, setCurrentPage] = useState(1);

  const totalPages = pageSize === Infinity ? 1 : Math.max(1, Math.ceil(data.length / pageSize));
  const startIndex = pageSize === Infinity ? 0 : (currentPage - 1) * pageSize;
  const endIndex = pageSize === Infinity ? data.length : Math.min(startIndex + pageSize, data.length);
  const pageData = isLoading ? [] : data.slice(startIndex, endIndex);

  const handleSort = (key: string) => {
    if (!sortable || !onSort) return;

    let newDirection: 'asc' | 'desc' = 'asc';
    if (sortKey === key) {
      newDirection = sortDirection === 'asc' ? 'desc' : 'asc';
    }
    setSortKey(key);
    setSortDirection(newDirection);
    onSort(key, newDirection);
  };

  const handlePageChange = (page: number) => {
    setCurrentPage(page);
  };

  // Loading state
  if (isLoading) {
    return (
      <div className={`overflow-x-auto rounded-lg border border-border ${className}`}>
        <table className="w-full border-collapse">
          <thead>
            <tr>
              {columns.map((col) => (
                <th key={col.key} className={thStyles} style={col.width ? { width: col.width } : undefined}>
                  {col.header}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {Array.from({ length: 5 }, (_, rowIdx) => (
              <tr key={rowIdx}>
                {columns.map((col) => (
                  <td key={col.key} className={tdStyles}>
                    <LoadingSkeleton variant="text" />
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    );
  }

  // Empty state
  if (data.length === 0 && emptyState) {
    return (
      <div className={`rounded-lg border border-border ${className}`}>
        <EmptyState
          icon={emptyState.icon}
          title={emptyState.title}
          message={emptyState.message}
          action={emptyState.action}
        />
      </div>
    );
  }

  return (
    <div className={`overflow-x-auto rounded-lg border border-border ${className}`}>
      <table className="w-full border-collapse">
        <thead>
          <tr>
            {columns.map((col) => (
              <th
                key={col.key}
                className={`${thStyles} ${sortable && col.sortable ? 'cursor-pointer select-none hover:bg-bg-hover' : ''}`}
                style={col.width ? { width: col.width } : undefined}
                onClick={() => col.sortable && handleSort(col.key)}
                aria-sort={
                  sortKey === col.key
                    ? sortDirection === 'asc'
                      ? 'ascending'
                      : 'descending'
                    : undefined
                }
              >
                <div className="flex items-center gap-1">
                  {col.header}
                  {sortable && col.sortable && sortKey === col.key && (
                    <span className="inline-flex">
                      {sortDirection === 'asc' ? (
                        <ArrowUp className="h-3 w-3" />
                      ) : (
                        <ArrowDown className="h-3 w-3" />
                      )}
                    </span>
                  )}
                </div>
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {pageData.map((item, rowIdx) => (
          <tr
            key={(item as Record<string, unknown>)?.id as string | number | undefined ?? rowIdx}
              className={`border-b border-border last:border-b-0 transition-colors ${onRowClick ? 'cursor-pointer hover:bg-bg-hover' : 'hover:bg-bg-hover'}`}
              onClick={() => onRowClick?.(item)}
            >
              {columns.map((col) => (
                <td key={col.key} className={tdStyles}>
                  {col.render(item)}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>

      {/* Pagination */}
      {totalPages > 1 && (
        <div className="flex items-center justify-between border-t border-border px-4 py-3">
          <button
            type="button"
            disabled={currentPage <= 1}
            onClick={() => handlePageChange(currentPage - 1)}
            className="inline-flex items-center gap-1 rounded-md px-3 py-1.5 text-sm font-medium text-text-secondary transition-colors hover:bg-bg-hover hover:text-text-primary disabled:pointer-events-none disabled:opacity-40"
          >
            <ChevronLeft className="h-4 w-4" />
            Previous
          </button>

          <span className="text-sm text-text-secondary">
            Page {currentPage} of {totalPages}
          </span>

          <button
            type="button"
            disabled={currentPage >= totalPages}
            onClick={() => handlePageChange(currentPage + 1)}
            className="inline-flex items-center gap-1 rounded-md px-3 py-1.5 text-sm font-medium text-text-secondary transition-colors hover:bg-bg-hover hover:text-text-primary disabled:pointer-events-none disabled:opacity-40"
          >
            Next
            <ChevronRight className="h-4 w-4" />
          </button>
        </div>
      )}
    </div>
  );
}
