import { useCallback, useState } from 'react';
import { Link } from 'react-router';
import {
  RefreshCw,
  TrendingUp,
  Zap,
  Database,
  Activity,
  Bot,
  Puzzle,
  DollarSign,
  Share2,
  ArrowRight,
} from 'lucide-react';
import {
  useEfficiencyOverview,
  useEfficiencySessions,
  useEfficiencyAgents,
  useEfficiencyTokens,
  useEfficiencySkills,
  useEfficiencyCorrelation,
} from '@/api/hooks';
import type {
  AgentPerformance,
  CorrelationMatrix,
  EfficiencyOverview,
  SkillEffectiveness,
} from '@/api/types';
import { PageHeader } from '@/components/layout/PageHeader';
import { StatCard, type Trend } from '@/components/ui/StatCard';
import { DataTable, type Column } from '@/components/ui/DataTable';
import { LoadingSkeleton } from '@/components/ui/LoadingSkeleton';
import { EmptyState } from '@/components/ui/EmptyState';
import { TimeframeFilter } from '@/components/ui/TimeframeFilter';
import { Button } from '@/components/ui/Button';
import { formatNumber } from '@/utils/formatters';

/* ─── Inline types ───────────────────────────────────────────── */
interface TokenUsageData {
  total_tokens: number;
  avg_per_session: number;
  by_model: Record<string, number>;
  daily: { date: string; tokens: number }[];
}

/* ─── Trend helper ─────────────────────────────────────────── */
function trendFromValue(value: number): Trend['direction'] {
  if (value > 0) return 'up';
  if (value < 0) return 'down';
  return 'neutral';
}

/* ─── Metric card component ──────────────────────────────────── */
interface MetricCardProps {
  icon: typeof Database;
  label: string;
  value: string;
  trend?: Trend;
  subtext?: string;
  to: string;
  color?: string;
  progress?: number; // 0-100 for progress bar
}

function MetricCard({ icon: Icon, label, value, trend, subtext, to, color = 'accent', progress }: MetricCardProps) {
  return (
    <Link
      to={to}
      className="group flex flex-col gap-3 rounded-lg border border-border bg-surface p-5 transition-colors hover:border-accent/30 hover:bg-surface-hover"
    >
      {/* Header with icon */}
      <div className="flex items-center justify-between">
        <div className={`flex h-9 w-9 items-center justify-center rounded-lg`}
          style={{ backgroundColor: `color-mix(in srgb, var(--color-${color}) 12%, transparent)` }}
        >
          <Icon className="h-4 w-4" style={{ color: `var(--color-${color})` }} aria-hidden="true" />
        </div>
        <ArrowRight className="h-4 w-4 text-text-tertiary opacity-0 transition-opacity group-hover:opacity-100" />
      </div>

      {/* Value */}
      <div className="flex items-baseline gap-2">
        <span className="text-2xl font-bold text-text-primary">{value}</span>
        {trend && (
          <span
            className={`text-xs font-medium ${
              trend.direction === 'up'
                ? 'text-success'
                : trend.direction === 'down'
                  ? 'text-error'
                  : 'text-text-tertiary'
            }`}
          >
            {trend.direction === 'up' ? '▲' : trend.direction === 'down' ? '▼' : '▬'}{' '}
            {trend.percentage}%
          </span>
        )}
      </div>

      {/* Label and subtext */}
      <div>
        <p className="text-sm font-medium text-text-primary">{label}</p>
        {subtext && <p className="mt-0.5 text-xs text-text-tertiary">{subtext}</p>}
      </div>

      {/* Progress bar (optional) */}
      {progress != null && (
        <div className="h-1.5 w-full overflow-hidden rounded-full bg-bg-tertiary">
          <div
            className="h-full rounded-full transition-all duration-500"
            style={{ width: `${Math.min(100, Math.max(0, progress))}%`, backgroundColor: `var(--color-${color})` }}
          />
        </div>
      )}
    </Link>
  );
}

/* ─── Skill columns ────────────────────────────────────────── */
const skillsColumns: Column<SkillEffectiveness>[] = [
  {
    key: 'skill_name',
    header: 'Skill',
    render: (s) => (
      <span className="font-medium text-text-primary">{s.skill_name}</span>
    ),
  },
  {
    key: 'effectiveness_score',
    header: 'Score',
    render: (s) => (
      <span className="text-text-secondary">{s.effectiveness_score}</span>
    ),
  },
  {
    key: 'usage_count',
    header: 'Usage',
    render: (s) => (
      <span className="text-text-secondary">{formatNumber(s.usage_count)}</span>
    ),
  },
  {
    key: 'trend',
    header: 'Trend',
    render: (s) => {
      const dir = trendFromValue(s.trend);
      const color =
        dir === 'up'
          ? 'text-success'
          : dir === 'down'
            ? 'text-error'
            : 'text-text-tertiary';
      const sign = s.trend > 0 ? '+' : '';
      return (
        <span className={`font-medium ${color}`}>
          {sign}
          {s.trend}%
        </span>
      );
    },
  },
];

/* ─── Component ────────────────────────────────────────────── */

export function EfficiencyPage() {
  const [timeframe, setTimeframe] = useState('30d');

  const overview = useEfficiencyOverview(timeframe);
  const sessions = useEfficiencySessions(timeframe);
  const agents = useEfficiencyAgents(timeframe);
  const tokens = useEfficiencyTokens(timeframe);
  const skills = useEfficiencySkills(timeframe);
  const correlation = useEfficiencyCorrelation(timeframe);

  /* ── Derived state ─────────────────────────────────────── */
  const isLoading =
    overview.isLoading ||
    sessions.isLoading ||
    agents.isLoading ||
    tokens.isLoading;

  const isError =
    overview.isError ||
    sessions.isError ||
    agents.isError ||
    tokens.isError;

  const overviewData: EfficiencyOverview | undefined = overview.data;
  const sessionsData = sessions.data ?? [];
  const agentsData: AgentPerformance[] = agents.data ?? [];
  const tokensData = tokens.data as TokenUsageData | undefined;
  const skillsData: SkillEffectiveness[] = skills.data ?? [];
  const correlationData: CorrelationMatrix | undefined = correlation.data;

  const noSkills = !skills.isLoading && skillsData.length === 0;

  /* ── Retry handler ──────────────────────────────────────── */
  const handleRetry = useCallback(() => {
    overview.refetch();
    sessions.refetch();
    agents.refetch();
    tokens.refetch();
    skills.refetch();
    correlation.refetch();
  }, [overview, sessions, agents, tokens, skills, correlation]);

  /* ── Render loading state ───────────────────────────────── */
  if (isLoading && !isError) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader title="Efficiency Mapper">
          <TimeframeFilter value={timeframe} onChange={setTimeframe} />
        </PageHeader>

        {/* Stat card skeletons */}
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <LoadingSkeleton variant="card" />
          <LoadingSkeleton variant="card" />
          <LoadingSkeleton variant="card" />
          <LoadingSkeleton variant="card" />
        </div>

        {/* Grid card skeletons */}
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
          <LoadingSkeleton variant="card" />
          <LoadingSkeleton variant="card" />
          <LoadingSkeleton variant="card" />
          <LoadingSkeleton variant="card" />
          <LoadingSkeleton variant="card" />
          <LoadingSkeleton variant="card" />
        </div>
      </div>
    );
  }

  /* ── Render error state ─────────────────────────────────── */
  if (isError) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader title="Efficiency Mapper">
          <TimeframeFilter value={timeframe} onChange={setTimeframe} />
        </PageHeader>

        <div className="flex flex-col items-center justify-center gap-4 rounded-lg border border-border bg-surface p-8 text-center">
          <div className="rounded-full bg-error/10 p-3">
            <TrendingUp className="h-6 w-6 text-error" aria-hidden="true" />
          </div>
          <h3 className="text-lg font-semibold text-text-primary">
            Failed to load efficiency data
          </h3>
          <p className="max-w-sm text-sm text-text-secondary">
            Something went wrong while fetching your data. Please try again.
          </p>
          <Button variant="primary" onClick={handleRetry}>
            <RefreshCw className="h-4 w-4" />
            Retry
          </Button>
        </div>
      </div>
    );
  }

  /* ── Data preparation ───────────────────────────────────── */
  const avgEfficiency = overviewData?.avg_efficiency ?? 0;
  const memUsage = overviewData?.memory_used_percent ?? 50;
  const avgTokensOverview = overviewData?.avg_tokens ?? 0;
  const avgTokens = tokensData?.avg_per_session ?? avgTokensOverview;
  const totalTokens = tokensData?.total_tokens ?? 0;
  const sessionCount = overviewData?.session_count ?? sessionsData.length;
  const avgAgent = agentsData.length > 0
    ? agentsData.reduce((sum, a) => sum + a.efficiency_score, 0) / agentsData.length
    : 0;
  const avgSkillEffectiveness = skillsData.length > 0
    ? skillsData.reduce((sum, s) => sum + s.effectiveness_score, 0) / skillsData.length
    : 0;
  const trend = overviewData?.trend ?? 0;
  const overallTrend: Trend = {
    direction: trendFromValue(trend),
    percentage: Math.abs(trend),
  };

  const avgDuration = overviewData?.avg_duration_minutes ?? 0;

  /* ── Render ──────────────────────────────────────────────── */
  return (
    <div className="flex flex-col gap-lg">
      {/* Header + Timeframe Filter */}
      <PageHeader title="Efficiency Mapper">
        <TimeframeFilter value={timeframe} onChange={setTimeframe} />
      </PageHeader>

      {/* Top stat cards row (4 compact stats) */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <StatCard
          value={`${avgEfficiency}%`}
          label="Avg Efficiency"
          trend={overallTrend}
        />
        <StatCard
          value={`${trend > 0 ? '+' : ''}${trend}%`}
          label="Trend"
          trend={overallTrend}
        />
        <StatCard
          value={formatNumber(Math.round(avgTokens))}
          label="Avg Tokens"
          trend={overallTrend}
        />
        <StatCard
          value={`${avgDuration}m`}
          label="Avg Duration"
          trend={overallTrend}
        />
      </div>

      {/* 3x2 detailed metric card grid */}
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
        <MetricCard
          icon={Database}
          label="Memory Usage"
          value={`${memUsage}%`}
          subtext="Used storage capacity"
          trend={{ direction: 'up', percentage: 5 }}
          progress={memUsage}
          to="/efficiency/memory"
          color="accent"
        />
        <MetricCard
          icon={Activity}
          label="Session Activity"
          value={formatNumber(sessionCount)}
          subtext="Total sessions in period"
          trend={{ direction: sessionCount > 0 ? 'up' : 'neutral', percentage: Math.min(sessionCount, 100) }}
          to="/efficiency/sessions"
          color="success"
        />
        <MetricCard
          icon={Bot}
          label="Agent Performance"
          value={`${Math.round(avgAgent)}%`}
          subtext={`${agentsData.length} agents tracked`}
          trend={{ direction: avgAgent > 70 ? 'up' : 'down', percentage: Math.round(Math.abs(avgAgent - 50)) }}
          progress={Math.round(avgAgent)}
          to="/efficiency/agents"
          color="accent"
        />
        <MetricCard
          icon={Puzzle}
          label="Skill Effectiveness"
          value={`${Math.round(avgSkillEffectiveness)}%`}
          subtext={`${skillsData.length} skills evaluated`}
          trend={{ direction: avgSkillEffectiveness > 70 ? 'up' : 'down', percentage: Math.round(Math.abs(avgSkillEffectiveness - 50)) }}
          progress={Math.round(avgSkillEffectiveness)}
          to="/efficiency/skills"
          color="info"
        />
        <MetricCard
          icon={DollarSign}
          label="Token Usage"
          value={formatNumber(totalTokens)}
          subtext={`${formatNumber(Math.round(avgTokens))} avg per session`}
          trend={{ direction: 'up', percentage: 15 }}
          to="/efficiency/tokens"
          color="warning"
        />
        <MetricCard
          icon={Share2}
          label="Correlation"
          value={correlationData ? `r=${correlationData.variables.length > 1 ? correlationData.correlations[0]?.[1]?.toFixed(2) ?? '0.00' : '0.00'}` : '—'}
          subtext={correlationData ? `${correlationData.variables.length} variables` : 'No data'}
          to="/efficiency/correlation"
          color="accent"
        />
      </div>

      {/* Skills Efficiency Table (existing detail section) */}
      <section className="flex flex-col gap-4">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold text-text-primary">
            Skills Efficiency
          </h2>
          <Link
            to="/efficiency/skills"
            className="text-sm font-medium text-accent transition-colors hover:text-accent-hover"
          >
            View Details &rarr;
          </Link>
        </div>

        {noSkills ? (
          <div className="rounded-lg border border-border">
            <EmptyState
              icon={Zap}
              title="No skill data available"
              message="Skills efficiency data will appear here once skills have been evaluated."
            />
          </div>
        ) : (
          <>
            <DataTable<SkillEffectiveness>
              columns={skillsColumns}
              data={skillsData}
              pageSize={10}
            />
            {correlationData && (
              <div className="mt-2">
                <CorrelationTable data={correlationData} />
              </div>
            )}
          </>
        )}
      </section>

      {/* Skills loading skeleton */}
      {skills.isLoading && (
        <div className="flex flex-col gap-4">
          <LoadingSkeleton variant="card" count={2} />
        </div>
      )}
    </div>
  );
}

/* ─── Correlation Table Sub-component ──────────────────────── */

function CorrelationTable({ data }: { data: CorrelationMatrix }) {
  return (
    <div className="flex flex-col gap-3">
      <h3 className="text-sm font-semibold text-text-primary">
        Correlation Matrix
      </h3>
      <div className="overflow-x-auto rounded-lg border border-border">
        <table className="w-full border-collapse text-sm">
          <thead>
            <tr>
              <th className="border-b border-border px-3 py-2 text-left text-xs font-semibold uppercase tracking-wider text-text-secondary">
                Variable
              </th>
              {data.variables.map((v) => (
                <th
                  key={v}
                  className="border-b border-border px-3 py-2 text-right text-xs font-semibold uppercase tracking-wider text-text-secondary"
                >
                  {v}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {data.variables.map((variable, rowIdx) => (
              <tr
                key={variable}
                className="border-b border-border last:border-b-0 hover:bg-bg-hover"
              >
                <td className="px-3 py-2 font-medium text-text-primary">
                  {variable}
                </td>
                {data.correlations[rowIdx]?.map((value, colIdx) => {
                  const abs = Math.abs(value);
                  const isSelf = rowIdx === colIdx;
                  const strength =
                    abs > 0.5
                      ? 'text-accent'
                      : abs > 0.3
                        ? 'text-text-primary'
                        : 'text-text-tertiary';

                  return (
                    <td
                      key={`${variable}-${data.variables[colIdx]}`}
                      className={`px-3 py-2 text-right font-mono text-xs ${isSelf ? 'font-bold' : ''} ${strength}`}
                    >
                      {value.toFixed(2)}
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
