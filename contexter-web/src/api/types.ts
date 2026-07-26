// ─── Session Domain ───────────────────────────────────────────────────────

export interface Session {
  id: string;
  project: string;
  agent: string;
  status: 'active' | 'done' | 'error' | 'paused';
  duration_minutes: number;
  turn_count: number;
  created_at: string;
  last_active: string;
}

export interface SessionDetail extends Session {
  turns: Turn[];
  memories_created: number;
  tokens_used: number;
  tags: string[];
}

export interface Turn {
  id: string;
  session_id: string;
  number: number;
  role: 'user' | 'agent';
  content: string;
  latency_ms?: number;
  agent?: string;
  created_at: string;
  tokens?: number;
  metadata?: Record<string, unknown>;
}

// ─── Memory Domain ────────────────────────────────────────────────────────

export interface Memory {
  id: string;
  content: string;
  memory_type: 'conversation' | 'decision' | 'pattern' | 'reference' | 'custom';
  tags: string[];
  source_session?: string;
  confidence: number;
  version: number;
  created_at: string;
  updated_at: string;
}

export interface MemoryDetail extends Memory {
  versions: MemoryVersion[];
  related_memories: { id: string; content: string; similarity: number }[];
  metadata: Record<string, unknown>;
}

export interface MemoryVersion {
  version: number;
  content: string;
  tags: string[];
  created_at: string;
}

// ─── Agent Domain ─────────────────────────────────────────────────────────

export interface Agent {
  id: string;
  name: string;
  capabilities: string[];
  status: 'active' | 'idle' | 'error' | 'offline';
  efficiency_score: number;
  sessions_count: number;
  avg_latency_ms: number;
  created_at: string;
  last_active: string;
}

export interface AgentDetail extends Agent {
  recent_sessions: Session[];
  efficiency_history: { date: string; score: number }[];
  settings: Record<string, unknown>;
}

// ─── Skill Domain ─────────────────────────────────────────────────────────

export interface Skill {
  id: string;
  name: string;
  category: string;
  effectiveness_score: number;
  usage_count: number;
  created_at: string;
  last_used: string;
}

export interface SkillDetail extends Skill {
  recent_sessions: Session[];
  effectiveness_history: { date: string; score: number }[];
}

// ─── Efficiency Domain ────────────────────────────────────────────────────

export interface EfficiencyOverview {
  avg_efficiency: number;
  trend: number;
  avg_tokens: number;
  avg_duration_minutes: number;
  memory_used_percent: number;
  session_count: number;
  agent_count: number;
  skill_count: number;
}

export interface EfficiencyDetail {
  date: string;
  score: number;
  tokens: number;
  sessions: number;
}

export interface AgentPerformance {
  agent_id: string;
  agent_name: string;
  efficiency_score: number;
  sessions_count: number;
  avg_latency_ms: number;
  trend: number;
}

export interface SkillEffectiveness {
  skill_id: string;
  skill_name: string;
  effectiveness_score: number;
  usage_count: number;
  trend: number;
}

export interface CorrelationMatrix {
  variables: string[];
  correlations: number[][];
}

export interface EfficiencyMemory {
  total_memories: number;
  avg_confidence: number;
  type_distribution: Record<string, number>;
}

export interface EfficiencyTokens {
  total_tokens: number;
  avg_per_session: number;
  by_model: Record<string, number>;
  daily: { date: string; tokens: number }[];
}

// ─── Analytics Domain ─────────────────────────────────────────────────────

export interface AnalyticsOverview {
  system_health: 'healthy' | 'degraded' | 'critical';
  uptime_percent: number;
  error_rate: number;
  avg_response_time_ms: number;
  active_sessions: number;
  memory_usage_percent: number;
  api_requests_total: number;
  cost_total: number;
}

export interface PerformanceTrend {
  date: string;
  response_time_ms: number;
  throughput: number;
  error_rate: number;
}

export interface ResourceUsage {
  cpu_percent: number;
  memory_percent: number;
  disk_percent: number;
  active_connections: number;
}

export interface CostBreakdown {
  total_cost: number;
  by_model: { model: string; cost: number; tokens: number; percentage: number }[];
  daily_costs: { date: string; cost: number }[];
}

export interface ModelCostDetail {
  model: string;
  total_cost: number;
  total_tokens: number;
  input_tokens: number;
  output_tokens: number;
  avg_cost_per_token: number;
  daily_breakdown: { date: string; tokens: number; cost: number }[];
}

export interface ServiceStatus {
  name: string;
  status: 'healthy' | 'degraded' | 'down';
  uptime_percent: number;
  latency_ms: number;
  last_checked: string;
}

// ─── Settings Domain ──────────────────────────────────────────────────────

export interface SettingsSection {
  key: string;
  label: string;
  settings: Record<string, unknown>;
}

export interface ProviderConfig {
  name: string;
  type: string;
  enabled: boolean;
  config: Record<string, string>;
}

// ─── Notification Domain ──────────────────────────────────────────────────

export interface Notification {
  id: string;
  type: 'info' | 'warning' | 'error' | 'success';
  title: string;
  message: string;
  read: boolean;
  created_at: string;
}

// ─── Feedback Domain ──────────────────────────────────────────────────────

export interface BugReport {
  id: string;
  title: string;
  description: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  status: 'open' | 'in-progress' | 'resolved' | 'closed';
  created_at: string;
}

export interface FeatureRequest {
  id: string;
  title: string;
  description: string;
  status: 'under-review' | 'planned' | 'in-progress' | 'shipped' | 'declined';
  votes: number;
  created_at: string;
}

export interface ChangelogEntry {
  version: string;
  date: string;
  changes: { type: 'added' | 'changed' | 'fixed' | 'removed'; description: string }[];
}

// ─── Search Domain ────────────────────────────────────────────────────────

export interface SearchResult {
  id: string;
  type: 'session' | 'memory' | 'agent' | 'skill';
  title: string;
  snippet: string;
  score: number;
}

// ─── Export Domain ────────────────────────────────────────────────────────

export interface ExportJob {
  id: string;
  type: 'sessions' | 'memories' | 'analytics';
  format: 'json' | 'csv';
  status: 'pending' | 'processing' | 'completed' | 'failed';
  created_at: string;
  completed_at?: string;
  download_url?: string;
  error?: string;
}

// ─── Audit Domain ─────────────────────────────────────────────────────────

export interface AuditEntry {
  id: string;
  action: string;
  entity_type: string;
  entity_id: string;
  changes: { field: string; old_value?: unknown; new_value?: unknown }[];
  performed_by: string;
  created_at: string;
}

// ─── Onboarding Domain ────────────────────────────────────────────────────

export interface OnboardingStatus {
  current_step: number;
  total_steps: number;
  completed: boolean;
  steps: { id: string; label: string; completed: boolean }[];
}

// ─── Correlation Domain ───────────────────────────────────────────────────

export interface CorrelationOverview {
  dataset_stats: { variable: string; mean: number; std: number; min: number; max: number }[];
  top_correlations: { variable_1: string; variable_2: string; r: number; p_value: number }[];
}

export interface CorrelationTimeline {
  date: string;
  correlations: { variable_1: string; variable_2: string; r: number }[];
}

export interface CorrelationCompare {
  groups: string[];
  metric: string;
  values: { group: string; mean: number; std: number; n: number }[];
  test: { type: string; statistic: number; p_value: number; significant: boolean };
}
