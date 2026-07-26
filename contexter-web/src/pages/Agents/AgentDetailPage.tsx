import { useState } from 'react';
import { useParams, useNavigate } from 'react-router';
import {
  Activity,
  ChartLine,
  Settings2,
  Table2,
  Users,
} from 'lucide-react';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from 'recharts';
import { useAgent } from '@/api/hooks';
import type { AgentDetail } from '@/api/types';
import { PageHeader } from '@/components/layout/PageHeader';
import { Badge } from '@/components/ui/Badge';
import { Tag } from '@/components/ui/Tag';
import { StatCard } from '@/components/ui/StatCard';
import { DataTable } from '@/components/ui/DataTable';
import type { Column } from '@/components/ui/DataTable';
import { TabBar } from '@/components/ui/TabBar';
import type { Tab } from '@/components/ui/TabBar';
import { LoadingSkeleton } from '@/components/ui/LoadingSkeleton';
import { EmptyState } from '@/components/ui/EmptyState';

/* ── Status → Badge variant ────────────────────────────────── */
const statusVariant: Record<
  AgentDetail['status'],
  'success' | 'pending' | 'error' | 'offline'
> = {
  active: 'success',
  idle: 'pending',
  error: 'error',
  offline: 'offline',
};

/* ── Tab definitions ────────────────────────────────────────── */
/* REQ-005.7: Tabs MUST be: Overview, Sessions, Skills, Version History */
const TABS: Tab[] = [
  { id: 'overview', label: 'Overview', icon: <Activity className="h-4 w-4" /> },
  { id: 'sessions', label: 'Sessions', icon: <Table2 className="h-4 w-4" /> },
  { id: 'skills', label: 'Skills', icon: <ChartLine className="h-4 w-4" /> },
  { id: 'version-history', label: 'Version History', icon: <Settings2 className="h-4 w-4" /> },
];

/* ── Session columns for the DataTable ──────────────────────── */
const sessionColumns: Column<AgentDetail['recent_sessions'][number]>[] = [
  {
    key: 'id',
    header: 'ID',
    render: (s) => (
      <span className="font-mono text-xs text-text-secondary">{s.id}</span>
    ),
    width: '140px',
  },
  {
    key: 'status',
    header: 'Status',
    render: (s) => (
      <Badge
        variant={
          s.status === 'done'
            ? 'success'
            : s.status === 'active'
              ? 'info'
              : s.status === 'error'
                ? 'error'
                : 'pending'
        }
        size="sm"
      >
        {s.status}
      </Badge>
    ),
    width: '100px',
  },
  {
    key: 'duration',
    header: 'Duration',
    render: (s) => `${s.duration_minutes}m`,
    width: '100px',
  },
  {
    key: 'turns',
    header: 'Turns',
    render: (s) => s.turn_count,
    width: '80px',
  },
  {
    key: 'project',
    header: 'Project',
    render: (s) => s.project,
  },
];

/* ── Loading state ──────────────────────────────────────────── */
function LoadingState() {
  return (
    <div className="flex flex-col gap-6">
      <div className="flex items-center gap-4">
        <LoadingSkeleton variant="avatar" />
        <div className="flex flex-1 flex-col gap-2">
          <LoadingSkeleton variant="text" count={1} />
          <LoadingSkeleton variant="text" count={1} />
        </div>
      </div>
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
        {Array.from({ length: 3 }, (_, i) => (
          <div key={i} className="rounded-lg border border-border bg-surface p-4">
            <LoadingSkeleton variant="text" count={2} />
          </div>
        ))}
      </div>
    </div>
  );
}

/* ── Overview Tab ───────────────────────────────────────────── */
function OverviewTab({ agent }: { agent: AgentDetail }) {
  return (
    <div className="flex flex-col gap-6">
      {/* Stats grid */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
        <StatCard value={agent.sessions_count} label="Sessions" />
        <StatCard value={`${agent.avg_latency_ms}ms`} label="Avg Latency" />
        <StatCard value={agent.capabilities.length} label="Capabilities" />
      </div>

      {/* Capabilities list */}
      <div>
        <h3 className="mb-3 text-sm font-semibold uppercase tracking-wider text-text-secondary">
          Capabilities
        </h3>
        <div className="flex flex-wrap gap-2">
          {agent.capabilities.map((cap) => (
            <Tag key={cap} label={cap} color="info" />
          ))}
        </div>
      </div>
    </div>
  );
}

/* ── Sessions Tab ───────────────────────────────────────────── */
function SessionsTab({ agent }: { agent: AgentDetail }) {
  return (
    <div className="flex flex-col gap-4">
      <h3 className="text-sm font-semibold uppercase tracking-wider text-text-secondary">
        Recent Sessions
      </h3>
      <DataTable
        columns={sessionColumns}
        data={agent.recent_sessions}
        emptyState={{
          icon: Table2,
          title: 'No sessions',
          message: 'This agent has no recent sessions.',
        }}
      />
    </div>
  );
}

/* ── Skills Tab ─────────────────────────────────────────────── */
function SkillsTab({ agent }: { agent: AgentDetail }) {
  const chartData = agent.efficiency_history.map((entry) => ({
    date: entry.date,
    score: entry.score,
  }));

  return (
    <div className="flex flex-col gap-4">
      <h3 className="text-sm font-semibold uppercase tracking-wider text-text-secondary">
        Efficiency Trend
      </h3>
      <div className="h-64 w-full">
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={chartData}>
            <CartesianGrid strokeDasharray="3 3" stroke="var(--color-border)" />
            <XAxis
              dataKey="date"
              stroke="var(--color-text-tertiary)"
              fontSize={12}
            />
            <YAxis
              domain={[0, 100]}
              stroke="var(--color-text-tertiary)"
              fontSize={12}
              unit="%"
            />
            <Tooltip
              contentStyle={{
                background: 'var(--color-surface)',
                border: '1px solid var(--color-border)',
                borderRadius: '8px',
                fontSize: '13px',
              }}
            />
            <Line
              type="monotone"
              dataKey="score"
              stroke="var(--color-accent)"
              strokeWidth={2}
              dot={{ r: 4, fill: 'var(--color-accent)' }}
              activeDot={{ r: 6 }}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}

/* ── Version History Tab ────────────────────────────────────── */
function VersionHistoryTab({ agent }: { agent: AgentDetail }) {
  const configEntries = Object.entries(agent.settings ?? {});

  return (
    <div className="flex flex-col gap-4">
      <h3 className="text-sm font-semibold uppercase tracking-wider text-text-secondary">
        Configuration
      </h3>
      {configEntries.length === 0 ? (
        <p className="text-sm text-text-tertiary">No configuration available.</p>
      ) : (
        <div className="overflow-x-auto rounded-lg border border-border">
          <table className="w-full border-collapse">
            <thead>
              <tr>
                <th className="border-b border-border px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-text-secondary">
                  Key
                </th>
                <th className="border-b border-border px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-text-secondary">
                  Value
                </th>
              </tr>
            </thead>
            <tbody>
              {configEntries.map(([key, value]) => (
                <tr
                  key={key}
                  className="border-b border-border last:border-b-0"
                >
                  <td className="px-4 py-3 text-sm font-medium text-text-primary">
                    {key}
                  </td>
                  <td className="px-4 py-3 text-sm text-text-secondary">
                    {typeof value === 'object' && value !== null
                      ? JSON.stringify(value)
                      : String(value)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

/* ── Efficiency score color ─────────────────────────────────── */
function scoreColor(score: number): string {
  if (score >= 80) return 'text-success';
  if (score >= 50) return 'text-warning';
  return 'text-error';
}

/* ── Page component ─────────────────────────────────────────── */
export function AgentDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data: agent, isLoading, isError, error } = useAgent(id ?? '');
  const [activeTab, setActiveTab] = useState('overview');

  /* Loading state */
  if (isLoading) {
    return (
      <div>
        <LoadingSkeleton variant="text" count={1} />
        <div className="mt-lg">
          <LoadingState />
        </div>
      </div>
    );
  }

  /* Error state */
  if (isError || !agent) {
    return (
      <div>
        <PageHeader title="Agent" breadcrumbs={[{ label: 'Agents', href: '/agents' }]} />
        <EmptyState
          icon={Users}
          title="Agent not found"
          message={
            error instanceof Error
              ? error.message
              : 'The requested agent could not be found.'
          }
          action={
            <button
              type="button"
              onClick={() => navigate('/agents')}
              className="inline-flex items-center gap-2 rounded-md bg-accent px-4 py-2 text-sm font-medium text-text-inverse transition-colors hover:bg-accent-hover"
            >
              Back to Agents
            </button>
          }
        />
      </div>
    );
  }

  return (
    <div>
      {/* Page header with breadcrumbs */}
      <PageHeader
        title={agent.name}
        breadcrumbs={[
          { label: 'Agents', href: '/agents' },
          { label: agent.name },
        ]}
      />

      {/* Info header: status + capabilities + efficiency */}
      <div className="mb-lg flex flex-col gap-4 rounded-lg border border-border bg-surface p-4 sm:flex-row sm:items-center sm:justify-between">
        {/* Left: name, status, capabilities */}
        <div className="flex flex-col gap-3">
          <div className="flex items-center gap-3">
            <h2 className="text-xl font-bold text-text-primary">
              {agent.name}
            </h2>
            <Badge variant={statusVariant[agent.status]} size="md" dot>
              {agent.status}
            </Badge>
          </div>
          <div className="flex flex-wrap gap-1.5">
            {agent.capabilities.map((cap) => (
              <Tag key={cap} label={cap} />
            ))}
          </div>
        </div>

        {/* Right: highlighted efficiency score */}
        <div className="flex shrink-0 flex-col items-center gap-1 rounded-lg bg-bg-tertiary px-6 py-3">
          <span className="text-xs font-medium uppercase tracking-wider text-text-secondary">
            Efficiency Score
          </span>
          <span
            className={`text-3xl font-bold ${scoreColor(agent.efficiency_score)}`}
          >
            {agent.efficiency_score}%
          </span>
        </div>
      </div>

      {/* Tab bar */}
      <div className="mb-lg">
        <TabBar
          tabs={TABS}
          activeTab={activeTab}
          onChange={setActiveTab}
        />
      </div>

      {/* Tab content */}
      {activeTab === 'overview' && <OverviewTab agent={agent} />}
      {activeTab === 'sessions' && <SessionsTab agent={agent} />}
      {activeTab === 'skills' && <SkillsTab agent={agent} />}
      {activeTab === 'version-history' && <VersionHistoryTab agent={agent} />}
    </div>
  );
}
