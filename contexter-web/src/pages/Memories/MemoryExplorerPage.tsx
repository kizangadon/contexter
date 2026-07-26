import { useState, useEffect, useMemo } from 'react';
import { useNavigate } from 'react-router';
import { Brain, Search as SearchIcon } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';
import { useMemories, useMemorySearch } from '@/api/hooks';
import type { Memory } from '@/api/types';
import { PageHeader } from '@/components/layout/PageHeader';
import { DataTable } from '@/components/ui/DataTable';
import { Badge, type BadgeVariant } from '@/components/ui/Badge';
import { Tag } from '@/components/ui/Tag';
import { Input } from '@/components/ui/Input';
import { FilterBar } from '@/components/ui/FilterBar';

/* ── Helpers ───────────────────────────────────────────────── */

const typeBadgeVariant: Record<string, BadgeVariant> = {
  conversation: 'info',
  decision: 'warning',
  pattern: 'success',
  reference: 'pending',
  custom: 'offline',
};

const TYPE_FILTER_OPTIONS = [
  { value: '', label: 'All' },
  { value: 'conversation', label: 'Conversation' },
  { value: 'decision', label: 'Decision' },
  { value: 'pattern', label: 'Pattern' },
  { value: 'reference', label: 'Reference' },
  { value: 'custom', label: 'Custom' },
];

function formatConfidence(value: number): string {
  return `${Math.round(value * 100)}%`;
}

function truncate(text: string, max: number): string {
  if (text.length <= max) return text;
  return text.slice(0, max).trimEnd() + '…';
}

/* ── Component ──────────────────────────────────────────────── */

export function MemoryExplorerPage() {
  const navigate = useNavigate();
  const [searchQuery, setSearchQuery] = useState('');
  const [debouncedSearch, setDebouncedSearch] = useState('');
  const [memoryTypeFilter, setMemoryTypeFilter] = useState('');
  const [sortKey, setSortKey] = useState<string | null>(null);
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('asc');

  // Debounce search input
  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedSearch(searchQuery);
    }, 300);
    return () => clearTimeout(timer);
  }, [searchQuery]);

  const isSearching = debouncedSearch.length >= 2;
  const { data: memories, isLoading: memoriesLoading } = useMemories(
    memoryTypeFilter ? { memory_type: memoryTypeFilter } : undefined,
  );
  const { data: searchResults, isLoading: searchLoading } = useMemorySearch(debouncedSearch);

  const isLoading = isSearching ? searchLoading : memoriesLoading;

  // Memoize rawData to avoid creating a new array reference on every render,
  // which would cause the sortedData useMemo to re-run unnecessarily.
  const rawData = useMemo(
    () => (isSearching ? (searchResults ?? []) : (memories ?? [])),
    [isSearching, searchResults, memories],
  );

  // Client-side sort
  const sortedData = useMemo(() => {
    if (!sortKey) return rawData;
    return [...rawData].sort((a, b) => {
      let cmp = 0;
      if (sortKey === 'confidence') {
        cmp = a.confidence - b.confidence;
      } else if (sortKey === 'version') {
        cmp = a.version - b.version;
      } else if (sortKey === 'updated') {
        cmp = new Date(a.updated_at).getTime() - new Date(b.updated_at).getTime();
      }
      return sortDirection === 'asc' ? cmp : -cmp;
    });
  }, [rawData, sortKey, sortDirection]);

  const handleSort = (key: string, direction: 'asc' | 'desc') => {
    setSortKey(key);
    setSortDirection(direction);
  };

  const handleRowClick = (memory: Memory) => {
    navigate(`/memories/${memory.id}`);
  };

  const handleSearchChange = (query: string) => {
    setSearchQuery(query);
  };

  return (
    <div>
      <PageHeader title="Memory Explorer" />

      <div className="mb-lg flex flex-col gap-lg">
        {/* Search input */}
        <Input
          icon={SearchIcon}
          placeholder="Search memories…"
          value={searchQuery}
          onChange={(e) => handleSearchChange(e.target.value)}
          aria-label="Search memories"
        />

        {/* Filter bar */}
        <FilterBar
          filters={[
            {
              key: 'memory_type',
              label: 'Memory Type',
              value: memoryTypeFilter,
              onChange: setMemoryTypeFilter,
              options: TYPE_FILTER_OPTIONS,
            },
          ]}
        />
      </div>

      <DataTable
        columns={columns}
        data={sortedData}
        isLoading={isLoading}
        sortable
        onSort={handleSort}
        onRowClick={handleRowClick}
        pageSize={10}
        emptyState={{
          icon: Brain,
          title: 'No memories found',
          message: 'Try adjusting your search query or filter.',
        }}
      />
    </div>
  );
}

/* ── Table columns ──────────────────────────────────────────── */

const columns = [
  {
    key: 'content',
    header: 'Content',
    render: (m: Memory) => (
      <span className="text-text-primary">{truncate(m.content, 80)}</span>
    ),
    width: '1fr',
  },
  {
    key: 'memory_type',
    header: 'Type',
    render: (m: Memory) => (
      <Badge variant={typeBadgeVariant[m.memory_type] ?? 'info'} size="sm">
        {m.memory_type}
      </Badge>
    ),
    width: '120px',
  },
  {
    key: 'tags',
    header: 'Tags',
    render: (m: Memory) => {
      const visible = m.tags.slice(0, 3);
      const remaining = m.tags.length - 3;
      return (
        <div className="flex flex-wrap gap-1">
          {visible.map((tag) => (
            <Tag key={tag} label={tag} />
          ))}
          {remaining > 0 && (
            <span className="text-xs text-text-tertiary">+{remaining} more</span>
          )}
        </div>
      );
    },
    width: '180px',
  },
  {
    key: 'confidence',
    header: 'Confidence',
    sortable: true,
    render: (m: Memory) => (
      <span className="font-mono text-sm text-text-primary">
        {formatConfidence(m.confidence)}
      </span>
    ),
    width: '100px',
  },
  {
    key: 'version',
    header: 'Version',
    sortable: true,
    render: (m: Memory) => (
      <span className="font-mono text-sm text-text-secondary">v{m.version}</span>
    ),
    width: '80px',
  },
  {
    key: 'updated',
    header: 'Updated',
    sortable: true,
    render: (m: Memory) => (
      <span className="text-sm text-text-secondary" title={m.updated_at}>
        {formatDistanceToNow(new Date(m.updated_at), { addSuffix: true })}
      </span>
    ),
    width: '140px',
  },
];
