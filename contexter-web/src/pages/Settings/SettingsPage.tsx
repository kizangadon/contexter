import { useState, useEffect, useCallback } from 'react';
import { useParams, Link } from 'react-router';
import {
  Settings,
  Cpu,
  Bell,
  Database,
  Key,
  Users,
  BarChart3,
  RefreshCw,
  Save,
  X,
  Eye,
  EyeOff,
} from 'lucide-react';
import { useSettings, useUpdateSettings } from '@/api/hooks';
import { PageHeader } from '@/components/layout/PageHeader';
import { Button } from '@/components/ui/Button';
import { EmptyState } from '@/components/ui/EmptyState';
import { LoadingSkeleton } from '@/components/ui/LoadingSkeleton';

/* ─── Settings section definitions ─────────────────────────── */

interface SettingsSectionDef {
  id: string;
  label: string;
  icon: typeof Settings;
}

/* ── Section labels MUST match REQ-007.2 in SPEC.md ───────── */
const SETTINGS_SECTIONS: SettingsSectionDef[] = [
  { id: 'general', label: 'General', icon: Settings },
  { id: 'storage', label: 'Storage', icon: Database },
  { id: 'mcp-server', label: 'MCP Server', icon: Cpu },
  { id: 'llm-providers', label: 'LLM Providers', icon: Key },
  { id: 'notifications', label: 'Notifications', icon: Bell },
  { id: 'agents-skills', label: 'Agents & Skills', icon: Users },
  { id: 'analytics', label: 'Analytics', icon: BarChart3 },
  { id: 'data-management', label: 'Data Management', icon: Database },
];

/* ─── Sidebar navigation ───────────────────────────────────── */

function SidebarNav({ activeSection }: { activeSection: string }) {
  return (
    <nav className="w-44 shrink-0" aria-label="Settings sections">
      <ul className="flex flex-col gap-1">
        {SETTINGS_SECTIONS.map((section) => {
          const Icon = section.icon;
          const isActive = section.id === activeSection;

          return (
            <li key={section.id}>
              <Link
                to={`/settings/${section.id}`}
                aria-current={isActive ? 'page' : undefined}
                className={`flex items-center gap-2.5 rounded-lg px-3 py-2 text-sm font-medium transition-colors duration-150 ${
                  isActive
                    ? 'bg-accent/10 text-accent'
                    : 'text-text-secondary hover:bg-bg-hover hover:text-text-primary'
                }`}
              >
                <Icon className="h-4 w-4" aria-hidden="true" />
                {section.label}
              </Link>
            </li>
          );
        })}
      </ul>
    </nav>
  );
}

/* ─── Settings field renderer ──────────────────────────────── */

function SettingsField({
  name,
  value,
  onChange,
}: {
  name: string;
  value: unknown;
  onChange: (name: string, value: unknown) => void;
}) {
  if (typeof value === 'boolean') {
    return (
      <div className="flex items-center justify-between rounded-lg border border-border bg-surface px-4 py-3">
        <label
          htmlFor={`field-${name}`}
          className="text-sm font-medium capitalize text-text-primary"
        >
          {name.replace(/_/g, ' ')}
        </label>
        <label className="relative inline-flex cursor-pointer items-center">
          <input
            id={`field-${name}`}
            type="checkbox"
            checked={value}
            onChange={(e) => onChange(name, e.target.checked)}
            className="peer sr-only"
            aria-label={name}
          />
          <div className="h-5 w-9 rounded-full bg-bg-tertiary after:absolute after:left-[2px] after:top-[2px] after:h-4 after:w-4 after:rounded-full after:bg-text-tertiary after:transition-all peer-checked:bg-accent peer-checked:after:translate-x-full peer-checked:after:bg-white" />
        </label>
      </div>
    );
  }

  if (typeof value === 'number') {
    return (
      <div className="flex items-center justify-between rounded-lg border border-border bg-surface px-4 py-3">
        <label
          htmlFor={`field-${name}`}
          className="text-sm font-medium capitalize text-text-primary"
        >
          {name.replace(/_/g, ' ')}
        </label>
        <input
          id={`field-${name}`}
          type="number"
          value={value}
          onChange={(e) => onChange(name, Number(e.target.value))}
          className="w-32 rounded-md border border-border bg-bg-primary px-3 py-1.5 text-sm text-text-primary focus:border-accent focus:outline-none"
        />
      </div>
    );
  }

  if (typeof value === 'string') {
    /* EC-023: Show/hide toggle for sensitive fields (API keys, passwords, tokens, secrets) */
    const isSensitive = /^(?:api[_-]?)?(?:key|secret|password|token)$/i.test(
      name.replace(/\s/g, '_'),
    );
    const [showValue, setShowValue] = useState(false);

    if (isSensitive) {
      return (
        <div className="flex items-center justify-between rounded-lg border border-border bg-surface px-4 py-3">
          <label
            htmlFor={`field-${name}`}
            className="text-sm font-medium capitalize text-text-primary"
          >
            {name.replace(/_/g, ' ')}
          </label>
          <div className="flex items-center gap-2">
            <input
              id={`field-${name}`}
              type={showValue ? 'text' : 'password'}
              value={value}
              onChange={(e) => onChange(name, e.target.value)}
              className="w-48 rounded-md border border-border bg-bg-primary px-3 py-1.5 text-sm text-text-primary focus:border-accent focus:outline-none"
            />
            <button
              type="button"
              onClick={() => setShowValue(!showValue)}
              aria-label={showValue ? 'Hide value' : 'Show value'}
              className="rounded-md p-1.5 text-text-secondary transition-colors hover:bg-bg-hover hover:text-text-primary"
            >
              {showValue ? (
                <EyeOff className="h-4 w-4" />
              ) : (
                <Eye className="h-4 w-4" />
              )}
            </button>
          </div>
        </div>
      );
    }

    return (
      <div className="flex items-center justify-between rounded-lg border border-border bg-surface px-4 py-3">
        <label
          htmlFor={`field-${name}`}
          className="text-sm font-medium capitalize text-text-primary"
        >
          {name.replace(/_/g, ' ')}
        </label>
        <input
          id={`field-${name}`}
          type="text"
          value={value}
          onChange={(e) => onChange(name, e.target.value)}
          className="w-48 rounded-md border border-border bg-bg-primary px-3 py-1.5 text-sm text-text-primary focus:border-accent focus:outline-none"
        />
      </div>
    );
  }

  // Complex types (objects, arrays) — read-only formatted display
  return (
    <div className="rounded-lg border border-border bg-surface px-4 py-3">
      <span className="mb-1 block text-sm font-medium capitalize text-text-primary">
        {name.replace(/_/g, ' ')}
      </span>
      <pre className="overflow-x-auto text-xs text-text-secondary">
        {JSON.stringify(value, null, 2)}
      </pre>
    </div>
  );
}

/* ─── Main page component ──────────────────────────────────── */

export function SettingsPage() {
  const { section } = useParams<{ section: string }>();

  const activeSection = section || 'general';

  const { data, isLoading, isError, refetch } = useSettings(activeSection);
  const updateSettings = useUpdateSettings();

  const [editedValues, setEditedValues] = useState<Record<string, unknown>>({});
  const [hasChanges, setHasChanges] = useState(false);

  // Sync local state when data loads or section changes
  useEffect(() => {
    if (data?.settings) {
      setEditedValues({ ...data.settings });
      setHasChanges(false);
    }
  }, [data]);

  const handleFieldChange = useCallback((name: string, value: unknown) => {
    setEditedValues((prev) => ({ ...prev, [name]: value }));
    setHasChanges(true);
  }, []);

  const handleSave = useCallback(() => {
    updateSettings.mutate(
      { section: activeSection, data: editedValues },
      { onSuccess: () => setHasChanges(false) },
    );
  }, [activeSection, editedValues, updateSettings]);

  const handleDiscard = useCallback(() => {
    if (data?.settings) {
      setEditedValues({ ...data.settings });
      setHasChanges(false);
    }
  }, [data]);

  /* ── Loading state ────────────────────────────────────── */
  if (isLoading) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader title="Settings" />
        <div className="flex gap-lg">
          <SidebarNav activeSection={activeSection} />
          <div className="min-w-0 flex-1">
            <div className="mb-4">
              <LoadingSkeleton variant="text" className="h-6 w-48" />
            </div>
            <div className="flex flex-col gap-3">
              <LoadingSkeleton variant="card" count={3} />
            </div>
          </div>
        </div>
      </div>
    );
  }

  /* ── Error state ──────────────────────────────────────── */
  if (isError || !data) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader title="Settings" />
        <div className="flex gap-lg">
          <SidebarNav activeSection={activeSection} />
          <div className="min-w-0 flex-1">
            <div className="rounded-lg border border-border">
              <EmptyState
                icon={RefreshCw}
                title="Failed to load settings"
                message="Could not load this settings section. Please try again."
                action={
                  <Button variant="primary" onClick={() => refetch()}>
                    <RefreshCw className="h-4 w-4" />
                    Retry
                  </Button>
                }
              />
            </div>
          </div>
        </div>
      </div>
    );
  }

  /* ── Content state ────────────────────────────────────── */
  const settingsKeys = Object.keys(data.settings ?? {});
  const isSaving = updateSettings.isPending;

  return (
    <div className="flex flex-col gap-lg">
      <PageHeader title="Settings" />
      <div className="flex gap-lg">
        <SidebarNav activeSection={activeSection} />

        <div className="min-w-0 flex-1">
          {/* Section heading */}
          <h2 className="mb-4 text-lg font-semibold text-text-primary">
            {data.label}
          </h2>

          {/* Empty state — no configurable settings */}
          {settingsKeys.length === 0 ? (
            <div className="rounded-lg border border-border">
              <EmptyState
                icon={Settings}
                title="No settings"
                message="This section has no configurable settings."
              />
            </div>
          ) : (
            <div className="flex flex-col gap-3">
              {/* Field list */}
              {settingsKeys.map((key) => (
                <SettingsField
                  key={key}
                  name={key}
                  value={editedValues[key] ?? data.settings[key]}
                  onChange={handleFieldChange}
                />
              ))}

              {/* Save / Discard actions */}
              {hasChanges && (
                <div className="flex items-center gap-3 pt-2">
                  <Button
                    variant="primary"
                    onClick={handleSave}
                    disabled={isSaving}
                  >
                    <Save className="h-4 w-4" />
                    {isSaving ? 'Saving...' : 'Save Changes'}
                  </Button>
                  <Button
                    variant="secondary"
                    onClick={handleDiscard}
                    disabled={isSaving}
                  >
                    <X className="h-4 w-4" />
                    Discard
                  </Button>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
