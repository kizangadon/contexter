import { useState } from 'react';
import { MessageSquare, Bug, Lightbulb, RefreshCw } from 'lucide-react';
import { useChangelog, useSubmitBugReport, useSubmitSuggestion } from '@/api/hooks';
import type { BugReport } from '@/api/types';
import { PageHeader } from '@/components/layout/PageHeader';
import { Button } from '@/components/ui/Button';
import { LoadingSkeleton } from '@/components/ui/LoadingSkeleton';

type Tab = 'changelog' | 'bug' | 'suggestion';

const tabs: { id: Tab; label: string; icon: typeof MessageSquare }[] = [
  { id: 'changelog', label: 'Changelog', icon: MessageSquare },
  { id: 'bug', label: 'Report Bug', icon: Bug },
  { id: 'suggestion', label: 'Suggest Feature', icon: Lightbulb },
];

export function FeedbackPage() {
  const [activeTab, setActiveTab] = useState<Tab>('changelog');

  const changelog = useChangelog();
  const submitBug = useSubmitBugReport();
  const submitSuggestion = useSubmitSuggestion();

  /* ── Bug form state ─────────────────────────────────────── */
  const [bugTitle, setBugTitle] = useState('');
  const [bugDescription, setBugDescription] = useState('');
  const [bugSeverity, setBugSeverity] = useState<BugReport['severity']>('medium');
  const [bugSuccess, setBugSuccess] = useState(false);

  /* ── Suggestion form state ──────────────────────────────── */
  const [sugTitle, setSugTitle] = useState('');
  const [sugDescription, setSugDescription] = useState('');
  const [sugCategory, setSugCategory] = useState('general');
  const [sugSuccess, setSugSuccess] = useState(false);

  const handleSubmitBug = () => {
    if (!bugTitle.trim() || !bugDescription.trim()) return;
    submitBug.mutate(
      { title: bugTitle, description: bugDescription, severity: bugSeverity },
      {
        onSuccess: () => {
          setBugTitle('');
          setBugDescription('');
          setBugSeverity('medium');
          setBugSuccess(true);
        },
      },
    );
  };

  const handleSubmitSuggestion = () => {
    if (!sugTitle.trim() || !sugDescription.trim()) return;
    submitSuggestion.mutate(
      { title: sugTitle, description: sugDescription },
      {
        onSuccess: () => {
          setSugTitle('');
          setSugDescription('');
          setSugCategory('general');
          setSugSuccess(true);
        },
      },
    );
  };

  return (
    <div className="flex flex-col gap-lg">
      <PageHeader title="Feedback" />

      {/* Tabs */}
      <div className="flex gap-1 rounded-lg border border-border bg-surface p-1" role="tablist">
        {tabs.map((tab) => {
          const Icon = tab.icon;
          const isActive = activeTab === tab.id;
          return (
            <button
              key={tab.id}
              role="tab"
              aria-selected={isActive}
              onClick={() => setActiveTab(tab.id)}
              className={`flex items-center gap-2 rounded-md px-4 py-2 text-sm font-medium transition-colors ${
                isActive
                  ? 'bg-accent text-text-inverse'
                  : 'text-text-secondary hover:text-text-primary'
              }`}
            >
              <Icon className="h-4 w-4" aria-hidden="true" />
              {tab.label}
            </button>
          );
        })}
      </div>

      {/* Tab Content */}
      <div role="tabpanel">
        {activeTab === 'changelog' && <ChangelogPanel changelog={changelog} />}
        {activeTab === 'bug' && (
          <BugForm
            title={bugTitle}
            description={bugDescription}
            severity={bugSeverity}
            isPending={submitBug.isPending}
            isSuccess={bugSuccess}
            onTitleChange={setBugTitle}
            onDescriptionChange={setBugDescription}
            onSeverityChange={setBugSeverity}
            onSubmit={handleSubmitBug}
          />
        )}
        {activeTab === 'suggestion' && (
          <SuggestionForm
            title={sugTitle}
            description={sugDescription}
            category={sugCategory}
            isPending={submitSuggestion.isPending}
            isSuccess={sugSuccess}
            onTitleChange={setSugTitle}
            onDescriptionChange={setSugDescription}
            onCategoryChange={setSugCategory}
            onSubmit={handleSubmitSuggestion}
          />
        )}
      </div>
    </div>
  );
}

/* ─── Changelog Panel ───────────────────────────────────────── */

function ChangelogPanel({ changelog }: { changelog: ReturnType<typeof useChangelog> }) {
  if (changelog.isLoading) {
    return (
      <div className="flex flex-col gap-4">
        <LoadingSkeleton variant="card" count={3} />
      </div>
    );
  }

  if (changelog.isError) {
    return (
      <div className="flex flex-col items-center justify-center gap-4 rounded-lg border border-border bg-surface p-8 text-center">
        <div className="rounded-full bg-error/10 p-3">
          <RefreshCw className="h-6 w-6 text-error" aria-hidden="true" />
        </div>
        <h3 className="text-lg font-semibold text-text-primary">Failed to load changelog</h3>
        <Button variant="primary" onClick={() => changelog.refetch()}>
          <RefreshCw className="h-4 w-4" />
          Retry
        </Button>
      </div>
    );
  }

  const entries = changelog.data ?? [];

  return (
    <div className="flex flex-col gap-6">
      {entries.map((entry) => (
        <div key={entry.version} className="rounded-lg border border-border bg-surface p-6">
          <div className="mb-3 flex items-baseline gap-3">
            <h3 className="text-lg font-semibold text-text-primary">v{entry.version}</h3>
            <span className="text-sm text-text-tertiary">{entry.date}</span>
          </div>
          <ul className="flex flex-col gap-2">
            {entry.changes.map((change, idx) => (
              <li key={idx} className="flex items-start gap-2 text-sm">
                <ChangeBadge type={change.type} />
                <span className="text-text-secondary">{change.description}</span>
              </li>
            ))}
          </ul>
        </div>
      ))}
    </div>
  );
}

const changeTypeColors: Record<string, string> = {
  added: 'text-success',
  changed: 'text-info',
  fixed: 'text-warning',
  removed: 'text-error',
};

function ChangeBadge({ type }: { type: string }) {
  return (
    <span className={`shrink-0 rounded px-1.5 py-0.5 text-xs font-semibold uppercase ${changeTypeColors[type] ?? 'text-text-tertiary'} bg-current/10`}>
      {type}
    </span>
  );
}

/* ─── Bug Form ──────────────────────────────────────────────── */

function BugForm({
  title, description, severity, isPending, isSuccess,
  onTitleChange, onDescriptionChange, onSeverityChange, onSubmit,
}: {
  title: string; description: string; severity: BugReport['severity'];
  isPending: boolean; isSuccess: boolean;
  onTitleChange: (v: string) => void; onDescriptionChange: (v: string) => void;
  onSeverityChange: (v: BugReport['severity']) => void; onSubmit: () => void;
}) {
  return (
    <div className="flex flex-col gap-4 rounded-lg border border-border bg-surface p-6">
      <h3 className="text-sm font-semibold text-text-primary">Report a Bug</h3>
      <div className="flex flex-col gap-4">
        <div>
          <label htmlFor="bug-title" className="mb-1 block text-sm font-medium text-text-primary">Title</label>
          <input
            id="bug-title"
            value={title}
            onChange={(e) => onTitleChange(e.target.value)}
            className="w-full rounded-md border border-border bg-bg-primary px-3 py-2 text-sm text-text-primary placeholder:text-text-tertiary focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent"
            placeholder="Brief title for the bug"
          />
        </div>
        <div>
          <label htmlFor="bug-description" className="mb-1 block text-sm font-medium text-text-primary">Description</label>
          <textarea
            id="bug-description"
            value={description}
            onChange={(e) => onDescriptionChange(e.target.value)}
            rows={4}
            className="w-full resize-none rounded-md border border-border bg-bg-primary px-3 py-2 text-sm text-text-primary placeholder:text-text-tertiary focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent"
            placeholder="Describe what happened"
          />
        </div>
        <div>
          <label htmlFor="bug-severity" className="mb-1 block text-sm font-medium text-text-primary">Severity</label>
          <select
            id="bug-severity"
            value={severity}
            onChange={(e) => onSeverityChange(e.target.value as BugReport['severity'])}
            className="w-full rounded-md border border-border bg-bg-primary px-3 py-2 text-sm text-text-primary focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent"
          >
            <option value="low">Low</option>
            <option value="medium">Medium</option>
            <option value="high">High</option>
            <option value="critical">Critical</option>
          </select>
        </div>
        <div className="flex items-center justify-between">
          {isSuccess && <p className="text-sm text-success">Bug report submitted successfully!</p>}
          <div className="ml-auto">
            <Button variant="primary" onClick={onSubmit} disabled={!title.trim() || !description.trim()} loading={isPending}>
              Submit Bug Report
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}

/* ─── Suggestion Form ───────────────────────────────────────── */

function SuggestionForm({
  title, description, category, isPending, isSuccess,
  onTitleChange, onDescriptionChange, onCategoryChange, onSubmit,
}: {
  title: string; description: string; category: string;
  isPending: boolean; isSuccess: boolean;
  onTitleChange: (v: string) => void; onDescriptionChange: (v: string) => void;
  onCategoryChange: (v: string) => void; onSubmit: () => void;
}) {
  return (
    <div className="flex flex-col gap-4 rounded-lg border border-border bg-surface p-6">
      <h3 className="text-sm font-semibold text-text-primary">Suggest a Feature</h3>
      <div className="flex flex-col gap-4">
        <div>
          <label htmlFor="sug-title" className="mb-1 block text-sm font-medium text-text-primary">Title</label>
          <input
            id="sug-title"
            value={title}
            onChange={(e) => onTitleChange(e.target.value)}
            className="w-full rounded-md border border-border bg-bg-primary px-3 py-2 text-sm text-text-primary placeholder:text-text-tertiary focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent"
            placeholder="Feature title"
          />
        </div>
        <div>
          <label htmlFor="sug-description" className="mb-1 block text-sm font-medium text-text-primary">Description</label>
          <textarea
            id="sug-description"
            value={description}
            onChange={(e) => onDescriptionChange(e.target.value)}
            rows={4}
            className="w-full resize-none rounded-md border border-border bg-bg-primary px-3 py-2 text-sm text-text-primary placeholder:text-text-tertiary focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent"
            placeholder="Describe your idea"
          />
        </div>
        <div>
          <label htmlFor="sug-category" className="mb-1 block text-sm font-medium text-text-primary">Category</label>
          <select
            id="sug-category"
            value={category}
            onChange={(e) => onCategoryChange(e.target.value)}
            className="w-full rounded-md border border-border bg-bg-primary px-3 py-2 text-sm text-text-primary focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent"
          >
            <option value="general">General</option>
            <option value="ui">UI / UX</option>
            <option value="performance">Performance</option>
            <option value="integration">Integration</option>
          </select>
        </div>
        <div className="flex items-center justify-between">
          {isSuccess && <p className="text-sm text-success">Suggestion submitted successfully!</p>}
          <div className="ml-auto">
            <Button variant="primary" onClick={onSubmit} disabled={!title.trim() || !description.trim()} loading={isPending}>
              Submit Suggestion
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
