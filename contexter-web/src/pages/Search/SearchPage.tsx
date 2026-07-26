import { useState } from 'react';
import { Search, RefreshCw } from 'lucide-react';
import { useSearch } from '@/api/hooks';
import type { SearchResult } from '@/api/types';
import { PageHeader } from '@/components/layout/PageHeader';
import { DataTable, type Column } from '@/components/ui/DataTable';
import { Button } from '@/components/ui/Button';
import { Badge } from '@/components/ui/Badge';

/* ─── Type badge variant ───────────────────────────────────── */
const typeVariant: Record<SearchResult['type'], 'info' | 'success' | 'pending' | 'offline'> = {
  session: 'info',
  memory: 'success',
  agent: 'pending',
  skill: 'offline',
};

/* ─── Table columns ─────────────────────────────────────────── */
const columns: Column<SearchResult>[] = [
  {
    key: 'type',
    header: 'Type',
    render: (r) => (
      <Badge variant={typeVariant[r.type]} size="sm">
        {r.type}
      </Badge>
    ),
  },
  {
    key: 'title',
    header: 'Title',
    render: (r) => (
      <span className="font-medium text-text-primary">{r.title}</span>
    ),
  },
  {
    key: 'snippet',
    header: 'Snippet',
    render: (r) => (
      <span className="text-sm text-text-secondary line-clamp-2">{r.snippet}</span>
    ),
  },
  {
    key: 'score',
    header: 'Score',
    render: (r) => (
      <span className="font-mono text-xs text-text-secondary">{(r.score * 100).toFixed(0)}%</span>
    ),
  },
];

/* ─── Component ─────────────────────────────────────────────── */

export function SearchPage() {
  const [query, setQuery] = useState('');
  const search = useSearch(query);
  const results = search.data ?? [];
  const isLoading = search.isLoading && query.length >= 2;
  const isError = search.isError;
  const hasSearched = query.length >= 2;

  const handleRetry = () => {
    search.refetch();
  };

  return (
    <div className="flex flex-col gap-lg">
      <PageHeader title="Search" />

      {/* Search input */}
      <div className="relative">
        <Search
          className="pointer-events-none absolute left-3 top-1/2 h-5 w-5 -translate-y-1/2 text-text-tertiary"
          aria-hidden="true"
        />
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search sessions, memories, agents, skills..."
          aria-label="Search query"
          className="w-full rounded-lg border border-border bg-surface py-3 pl-10 pr-4 text-sm text-text-primary placeholder:text-text-tertiary focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent"
        />
      </div>

      {/* Error State */}
      {isError && (
        <div className="flex flex-col items-center justify-center gap-4 rounded-lg border border-border bg-surface p-8 text-center">
          <div className="rounded-full bg-error/10 p-3">
            <RefreshCw className="h-6 w-6 text-error" aria-hidden="true" />
          </div>
          <h3 className="text-lg font-semibold text-text-primary">Search failed</h3>
          <p className="max-w-sm text-sm text-text-secondary">
            Something went wrong while searching. Please try again.
          </p>
          <Button variant="primary" onClick={handleRetry}>
            <RefreshCw className="h-4 w-4" />
            Retry
          </Button>
        </div>
      )}

      {/* Results */}
      {!isError && (
        <DataTable<SearchResult>
          columns={columns}
          data={results}
          isLoading={isLoading}
          emptyState={{
            icon: Search,
            title: hasSearched ? 'No results found' : 'Start searching',
            message: hasSearched
              ? 'No results match your query. Try different keywords.'
              : 'Type at least 2 characters to search across sessions, memories, agents, and skills.',
          }}
        />
      )}
    </div>
  );
}
