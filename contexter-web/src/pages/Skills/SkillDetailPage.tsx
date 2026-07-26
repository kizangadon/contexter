import { useState } from 'react';
import { useParams, useNavigate } from 'react-router';
import { ArrowLeft, Calendar, TrendingUp, Activity } from 'lucide-react';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from 'recharts';
import { useSkill } from '@/api/hooks';
import { PageHeader } from '@/components/layout/PageHeader';
import { Badge } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import { DataTable, type Column } from '@/components/ui/DataTable';
import { EmptyState } from '@/components/ui/EmptyState';
import { LoadingSkeleton } from '@/components/ui/LoadingSkeleton';
import { StatCard } from '@/components/ui/StatCard';
import type { Session } from '@/api/types';

type Tab = 'overview' | 'usage' | 'versions';

const tabStyles = {
  base: 'px-4 py-2 text-sm font-medium transition-colors duration-150',
  active: 'border-b-2 border-accent text-accent',
  inactive: 'border-b-2 border-transparent text-text-secondary hover:text-text-primary hover:border-border-hover',
};

const sessionColumns: Column<Session>[] = [
  { key: 'id', header: 'ID', render: (s) => <span className="font-mono text-xs">{s.id}</span> },
  { key: 'status', header: 'Status', render: (s) => <Badge variant={s.status === 'active' ? 'success' : s.status === 'error' ? 'error' : 'info'} size="sm">{s.status}</Badge> },
  { key: 'duration', header: 'Duration', render: (s) => `${s.duration_minutes}m` },
  { key: 'turns', header: 'Turns', render: (s) => s.turn_count },
  { key: 'project', header: 'Project', render: (s) => s.project },
];

export function SkillDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [activeTab, setActiveTab] = useState<Tab>('overview');

  const { data: skill, isLoading, isError } = useSkill(id ?? '');

  // Loading state
  if (isLoading) {
    return (
      <div className="flex flex-col gap-lg">
        <LoadingSkeleton variant="text" count={1} className="h-6 w-48" />
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
          <LoadingSkeleton variant="card" count={3} />
        </div>
        <LoadingSkeleton variant="card" count={1} className="h-64" />
      </div>
    );
  }

  // Error state
  if (isError || !skill) {
    return (
      <EmptyState
        icon={ArrowLeft}
        title="Skill not found"
        message="The skill you are looking for does not exist or may have been removed."
        action={
          <Button variant="secondary" onClick={() => navigate('/skills')}>
            Back to Skills
          </Button>
        }
      />
    );
  }

  const tabs: { key: Tab; label: string; icon: typeof TrendingUp }[] = [
    { key: 'overview', label: 'Overview', icon: Activity },
    { key: 'usage', label: 'Usage', icon: TrendingUp },
    { key: 'versions', label: 'Versions', icon: Calendar },
  ];

  return (
    <div>
      {/* Page header with breadcrumbs */}
      <PageHeader
        title={skill.name}
        breadcrumbs={[
          { label: 'Skills', href: '/skills' },
          { label: skill.name },
        ]}
      />

      {/* Info cards */}
      <div className="mb-lg grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        <StatCard
          value={`${skill.effectiveness_score}%`}
          label="Effectiveness Score"
        />
        <StatCard
          value={skill.usage_count}
          label="Usage Count"
        />
        <div className="flex flex-col gap-1 rounded-lg border border-border bg-surface p-4">
          <span className="text-sm text-text-secondary">Category</span>
          <Badge variant="info">{skill.category}</Badge>
        </div>
      </div>

      {/* Tab bar */}
      <div className="mb-lg flex border-b border-border" role="tablist" aria-label="Skill details tabs">
        {tabs.map((tab) => {
          const isActive = activeTab === tab.key;
          return (
            <button
              key={tab.key}
              role="tab"
              aria-selected={isActive}
              onClick={() => setActiveTab(tab.key)}
              className={`${tabStyles.base} ${isActive ? tabStyles.active : tabStyles.inactive} flex items-center gap-2`}
            >
              <tab.icon className="h-4 w-4" aria-hidden="true" />
              {tab.label}
            </button>
          );
        })}
      </div>

      {/* Tab content */}
      <div role="tabpanel" aria-label={`${activeTab} content`}>
        {/* Overview tab */}
        {activeTab === 'overview' && (
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
            <div className="rounded-lg border border-border bg-surface p-4">
              <h3 className="mb-2 text-sm font-semibold text-text-secondary">Category</h3>
              <Badge variant="info">{skill.category}</Badge>
            </div>
            <div className="rounded-lg border border-border bg-surface p-4">
              <h3 className="mb-2 text-sm font-semibold text-text-secondary">Created</h3>
              <p className="text-sm text-text-primary">
                {new Date(skill.created_at).toLocaleDateString('en-US', {
                  year: 'numeric',
                  month: 'long',
                  day: 'numeric',
                })}
              </p>
            </div>
          </div>
        )}

        {/* Usage tab */}
        {activeTab === 'usage' && (
          <div className="rounded-lg border border-border bg-surface p-4">
            <h3 className="mb-4 text-sm font-semibold text-text-secondary">Usage Trend</h3>
            {skill.effectiveness_history && skill.effectiveness_history.length > 0 ? (
              <div className="h-64">
                <ResponsiveContainer width="100%" height="100%">
                  <LineChart data={skill.effectiveness_history}>
                    <CartesianGrid strokeDasharray="3 3" stroke="var(--color-border)" />
                    <XAxis
                      dataKey="date"
                      tick={{ fill: 'var(--color-text-secondary)', fontSize: 12 }}
                      tickLine={false}
                    />
                    <YAxis
                      domain={[0, 100]}
                      tick={{ fill: 'var(--color-text-secondary)', fontSize: 12 }}
                      tickLine={false}
                      axisLine={false}
                    />
                    <Tooltip
                      contentStyle={{
                        backgroundColor: 'var(--color-surface)',
                        border: '1px solid var(--color-border)',
                        borderRadius: '8px',
                      }}
                    />
                    <Line
                      type="monotone"
                      dataKey="score"
                      stroke="var(--color-accent)"
                      strokeWidth={2}
                      dot={{ fill: 'var(--color-accent)', r: 4 }}
                      activeDot={{ r: 6 }}
                    />
                  </LineChart>
                </ResponsiveContainer>
              </div>
            ) : (
              <p className="text-sm text-text-tertiary">No effectiveness data available.</p>
            )}
          </div>
        )}

        {/* Versions tab */}
        {activeTab === 'versions' && (
          <DataTable<Session>
            columns={sessionColumns}
            data={skill.recent_sessions ?? []}
            emptyState={{
              icon: Calendar,
              title: 'No sessions',
              message: 'This skill has not been used in any sessions yet.',
            }}
          />
        )}
      </div>
    </div>
  );
}
