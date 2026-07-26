import { useState } from 'react';
import { useNavigate } from 'react-router';
import { Users } from 'lucide-react';
import { useAgents } from '@/api/hooks';
import { PageHeader } from '@/components/layout/PageHeader';
import { FilterBar } from '@/components/ui/FilterBar';
import type { FilterDef } from '@/components/ui/FilterBar';
import { LoadingSkeleton } from '@/components/ui/LoadingSkeleton';
import { EmptyState } from '@/components/ui/EmptyState';
import { AgentCard } from './components/AgentCard';
import type { Agent } from '@/api/types';

const STATUS_OPTIONS = [
  { value: '', label: 'All' },
  { value: 'active', label: 'Active' },
  { value: 'idle', label: 'Idle' },
  { value: 'error', label: 'Error' },
  { value: 'offline', label: 'Offline' },
] as const;

export function AgentRegistryPage() {
  const navigate = useNavigate();
  const [statusFilter, setStatusFilter] = useState('');
  const { data: agents, isLoading } = useAgents(
    statusFilter ? { status: statusFilter } : undefined,
  );

  const filters: FilterDef[] = [
    {
      key: 'status',
      label: 'Status',
      options: [...STATUS_OPTIONS],
      value: statusFilter,
      onChange: setStatusFilter,
    },
  ];

  return (
    <div>
      <PageHeader title="Agents">
        {/* FilterBar is rendered inline below the title */}
      </PageHeader>

      <div className="mb-lg">
        <FilterBar filters={filters} />
      </div>

      {/* Loading state: skeleton grid */}
      {isLoading && (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {Array.from({ length: 6 }, (_, i) => (
            <div
              key={i}
              className="flex flex-col gap-3 rounded-lg border border-border bg-surface p-4"
            >
              <LoadingSkeleton variant="text" count={1} />
              <LoadingSkeleton variant="text" count={2} />
              <LoadingSkeleton variant="text" count={1} />
            </div>
          ))}
        </div>
      )}

      {/* Empty state */}
      {!isLoading && agents && agents.length === 0 && (
        <EmptyState
          icon={Users}
          title="No agents found"
          message="No agents match the current filter. Try adjusting your status filter."
        />
      )}

      {/* Agent grid */}
      {!isLoading && agents && agents.length > 0 && (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {agents.map((agent: Agent) => (
            <AgentCard
              key={agent.id}
              agent={agent}
              onClick={() => navigate(`/agents/${agent.id}`)}
            />
          ))}
        </div>
      )}
    </div>
  );
}
