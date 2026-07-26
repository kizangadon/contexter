import { useState } from 'react';
import { useParams, Link } from 'react-router';
import { format, formatDistanceToNow } from 'date-fns';
import { FileQuestion, AlertCircle, ArrowLeft } from 'lucide-react';
import { useMemory } from '@/api/hooks';
import type { MemoryDetail } from '@/api/types';
import { PageHeader } from '@/components/layout/PageHeader';
import { TabBar } from '@/components/ui/TabBar';
import { Badge, type BadgeVariant } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import { Tag } from '@/components/ui/Tag';
import { LoadingSkeleton } from '@/components/ui/LoadingSkeleton';
import { EmptyState } from '@/components/ui/EmptyState';
import { EntityLink } from '@/components/ui/EntityLink';

/* ── Helpers ───────────────────────────────────────────────── */

const typeBadgeVariant: Record<string, BadgeVariant> = {
  conversation: 'info',
  decision: 'warning',
  pattern: 'success',
  reference: 'pending',
  custom: 'offline',
};

function confidenceColor(value: number): string {
  if (value >= 0.7) return 'bg-success';
  if (value >= 0.3) return 'bg-warning';
  return 'bg-error';
}

function confidenceTextColor(value: number): string {
  if (value >= 0.7) return 'text-success';
  if (value >= 0.3) return 'text-warning';
  return 'text-error';
}

/* ── Component ──────────────────────────────────────────────── */

export function MemoryDetailPage() {
  const { id } = useParams();
  const resolvedId = id ?? '';
  const { data: memory, isLoading, isError, error, refetch } = useMemory(resolvedId);
  const [activeTab, setActiveTab] = useState('content');

  const is404 = isError && error && 'status' in error && (error as { status: number }).status === 404;

  // Loading state
  if (isLoading) {
    return (
      <div>
        <PageHeader title="Memory" />
        <div className="flex flex-col gap-lg">
          <LoadingSkeleton variant="card" count={1} />
          <LoadingSkeleton variant="text" count={4} />
        </div>
      </div>
    );
  }

  // 404 not found
  if (is404 || (!isLoading && !memory)) {
    return (
      <div>
        <EmptyState
          icon={FileQuestion}
          title="Memory not found"
          message="The memory you're looking for doesn't exist or has been removed."
          action={
            <Link
              to="/memories"
              className="inline-flex items-center gap-2 rounded-md bg-accent px-4 py-2 text-sm font-medium text-text-inverse transition-colors hover:bg-accent-hover"
            >
              <ArrowLeft className="h-4 w-4" />
              Back to memories
            </Link>
          }
        />
      </div>
    );
  }

  // Error state (non-404)
  if (isError) {
    return (
      <div>
        <EmptyState
          icon={AlertCircle}
          title="Failed to load memory"
          message="An error occurred while loading this memory. Please try again."
          action={
            <Button variant="primary" onClick={() => refetch()}>
              Retry
            </Button>
          }
        />
      </div>
    );
  }

  if (!memory) return null;

  return (
    <div>
      <PageHeader
        title="Memory"
        breadcrumbs={[
          { label: 'Memories', href: '/memories' },
          { label: `Memory ${resolvedId.slice(0, 11)}…` },
        ]}
      />

      <div className="grid grid-cols-1 gap-lg lg:grid-cols-3">
        {/* Main content area */}
        <div className="lg:col-span-2">
          {/* Content panel */}
          <div className="rounded-lg border border-border bg-bg-secondary p-lg">
            <p className="whitespace-pre-wrap text-sm leading-relaxed text-text-primary">
              {memory.content}
            </p>
          </div>

          {/* Tab bar */}
          <div className="mt-lg">
            <TabBar
              tabs={[
                { id: 'content', label: 'Content' },
                { id: 'versions', label: 'Versions' },
                { id: 'related', label: 'Related' },
              ]}
              activeTab={activeTab}
              onChange={setActiveTab}
            />

            <div className="mt-lg">
              {activeTab === 'content' && <ContentTab memory={memory} />}
              {activeTab === 'versions' && <VersionsTab memory={memory} />}
              {activeTab === 'related' && <RelatedTab memory={memory} />}
            </div>
          </div>
        </div>

        {/* Sidebar metadata */}
        <div className="flex flex-col gap-lg">
          <MetadataCard memory={memory} />
        </div>
      </div>
    </div>
  );
}

/* ── Metadata Card ──────────────────────────────────────────── */

function MetadataCard({ memory }: { memory: MemoryDetail }) {
  return (
    <div className="rounded-lg border border-border bg-bg-secondary p-lg">
      <h2 className="mb-md text-sm font-semibold uppercase tracking-wider text-text-secondary">
        Metadata
      </h2>

      <div className="flex flex-col gap-md">
        {/* Type */}
        <div>
          <span className="text-xs font-medium text-text-tertiary">Type</span>
          <div className="mt-1">
            <Badge variant={typeBadgeVariant[memory.memory_type] ?? 'info'} size="sm">
              {memory.memory_type}
            </Badge>
          </div>
        </div>

        {/* Confidence */}
        <div>
          <span className="text-xs font-medium text-text-tertiary">Confidence</span>
          <div className="mt-1 flex items-center gap-2">
            <div className="h-2 flex-1 overflow-hidden rounded-full bg-bg-tertiary">
              <div
                className={`h-full rounded-full transition-all ${confidenceColor(memory.confidence)}`}
                style={{ width: `${Math.round(memory.confidence * 100)}%` }}
              />
            </div>
            <span className={`text-xs font-medium ${confidenceTextColor(memory.confidence)}`}>
              {Math.round(memory.confidence * 100)}%
            </span>
          </div>
        </div>

        {/* Version */}
        <div>
          <span className="text-xs font-medium text-text-tertiary">Version</span>
          <p className="mt-1 font-mono text-sm text-text-primary">v{memory.version}</p>
        </div>

        {/* Source session — EC-013: show "(deleted)" instead of hiding */}
        <div>
          <span className="text-xs font-medium text-text-tertiary">Source Session</span>
          <div className="mt-1">
            {memory.source_session ? (
              <EntityLink to={`/sessions/${memory.source_session}`} type="session">
                {memory.source_session}
              </EntityLink>
            ) : (
              <span className="text-sm italic text-text-tertiary">(deleted)</span>
            )}
          </div>
        </div>

        {/* Tags */}
        {memory.tags.length > 0 && (
          <div>
            <span className="text-xs font-medium text-text-tertiary">Tags</span>
            <div className="mt-1 flex flex-wrap gap-1">
              {memory.tags.map((tag) => (
                <Tag key={tag} label={tag} />
              ))}
            </div>
          </div>
        )}

        {/* Created */}
        <div>
          <span className="text-xs font-medium text-text-tertiary">Created</span>
          <p className="mt-1 text-sm text-text-primary" title={memory.created_at}>
            {format(new Date(memory.created_at), 'MMM d, yyyy HH:mm')}
          </p>
          <p className="text-xs text-text-tertiary">
            {formatDistanceToNow(new Date(memory.created_at), { addSuffix: true })}
          </p>
        </div>

        {/* Updated */}
        <div>
          <span className="text-xs font-medium text-text-tertiary">Updated</span>
          <p className="mt-1 text-sm text-text-primary" title={memory.updated_at}>
            {format(new Date(memory.updated_at), 'MMM d, yyyy HH:mm')}
          </p>
          <p className="text-xs text-text-tertiary">
            {formatDistanceToNow(new Date(memory.updated_at), { addSuffix: true })}
          </p>
        </div>
      </div>
    </div>
  );
}

/* ── Content Tab ────────────────────────────────────────────── */

function ContentTab({ memory }: { memory: MemoryDetail }) {
  return (
    <div className="rounded-lg border border-border bg-bg-secondary p-lg">
      <h3 className="mb-sm text-sm font-semibold text-text-primary">Full Content</h3>
      <div className="max-h-96 overflow-y-auto">
        <p className="whitespace-pre-wrap text-sm leading-relaxed text-text-primary">
          {memory.content}
        </p>
      </div>
    </div>
  );
}

/* ── Versions Tab ───────────────────────────────────────────── */

function VersionsTab({ memory }: { memory: MemoryDetail }) {
  const [selectedVersion, setSelectedVersion] = useState<number | null>(null);

  if (memory.versions.length === 0) {
    return (
      <EmptyState
        title="No versions"
        message="This memory has no previous versions."
      />
    );
  }

  return (
    <div className="flex flex-col gap-md">
      {memory.versions.toReversed().map((v) => (
        <div
          key={v.version}
          className={`cursor-pointer rounded-lg border p-md transition-colors hover:bg-bg-hover ${
            selectedVersion === v.version ? 'border-accent' : 'border-border'
          }`}
          onClick={() => setSelectedVersion(v.version === selectedVersion ? null : v.version)}
          role="button"
          tabIndex={0}
          onKeyDown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              setSelectedVersion(v.version === selectedVersion ? null : v.version);
            }
          }}
          aria-expanded={selectedVersion === v.version}
        >
          <div className="mb-1 flex items-center gap-2">
            <span className="font-mono text-sm font-semibold text-text-primary">
              v{v.version}
            </span>
            <span className="text-xs text-text-tertiary">
              {formatDistanceToNow(new Date(v.created_at), { addSuffix: true })}
            </span>
          </div>

          <p className="line-clamp-2 text-sm text-text-secondary">
            {v.content}
          </p>

          {v.tags.length > 0 && (
            <div className="mt-2 flex flex-wrap gap-1">
              {v.tags.map((tag) => (
                <Tag key={tag} label={tag} />
              ))}
            </div>
          )}

          {selectedVersion === v.version && (
            <div className="mt-md rounded-md bg-bg-tertiary p-md">
              <p className="whitespace-pre-wrap text-sm leading-relaxed text-text-primary">
                {v.content}
              </p>
            </div>
          )}
        </div>
      ))}
    </div>
  );
}

/* ── Related Tab ────────────────────────────────────────────── */

function RelatedTab({ memory }: { memory: MemoryDetail }) {
  if (memory.related_memories.length === 0) {
    return (
      <EmptyState
        title="No related memories"
        message="No related memories found for this entry."
      />
    );
  }

  return (
    <div className="flex flex-col gap-md">
      {memory.related_memories.map((related) => (
        <div
          key={related.id}
          className="rounded-lg border border-border bg-bg-secondary p-md"
        >
          <div className="mb-1 flex items-center justify-between">
            <EntityLink to={`/memories/${related.id}`} type="memory">
              {related.id}
            </EntityLink>
            <span className="text-xs font-medium text-text-tertiary">
              {Math.round(related.similarity * 100)}% similar
            </span>
          </div>
          <p className="line-clamp-2 text-sm text-text-secondary">
            {related.content}
          </p>
        </div>
      ))}
    </div>
  );
}
