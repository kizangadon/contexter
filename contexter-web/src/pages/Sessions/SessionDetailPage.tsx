import { useState, useRef, useEffect } from 'react';
import { useParams, useNavigate, Link } from 'react-router';
import { Activity, Play, RefreshCw, Trash2, MoreVertical } from 'lucide-react';
import { useSession, useDeleteSession, useResumeSession } from '@/api/hooks';
import type { SessionDetail, Turn } from '@/api/types';
import { PageHeader } from '@/components/layout/PageHeader';
import { Badge } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import { TabBar, type Tab } from '@/components/ui/TabBar';
import { Tag } from '@/components/ui/Tag';
import { Modal } from '@/components/ui/Modal';
import { LoadingSkeleton } from '@/components/ui/LoadingSkeleton';
import { EmptyState } from '@/components/ui/EmptyState';
import { TurnTimeline } from '@/components/sessions/TurnTimeline';
import { MessageBubble } from './components/MessageBubble';
import { SessionInfoHeader } from './components/SessionInfoHeader';

/* ── Tab definitions ────────────────────────────────────────── */
const TABS: Tab[] = [
  { id: 'timeline', label: 'Timeline' },
  { id: 'messages', label: 'Messages' },
  { id: 'memories', label: 'Memories' },
  { id: 'metadata', label: 'Metadata' },
];

/* ── Messages Tab ───────────────────────────────────────────── */
function MessagesTab({ turns }: { turns: Turn[] }) {
  return (
    <div className="flex flex-col gap-3">
      {turns.length === 0 ? (
        <p className="py-8 text-center text-sm text-text-tertiary">
          No messages in this session.
        </p>
      ) : (
        turns.map((turn, index) => (
          <MessageBubble
            key={turn.id}
            turn={turn}
            isUser={turn.role === 'user'}
            turnNumber={index + 1}
          />
        ))
      )}
    </div>
  );
}

/* ── Memories Tab ───────────────────────────────────────────── */
function MemoriesTab({ session }: { session: SessionDetail }) {
  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center gap-2">
        <h3 className="text-sm font-semibold uppercase tracking-wider text-text-secondary">
          Memory Tags
        </h3>
        <Badge variant="info" size="sm">
          {session.memories_created} memories
        </Badge>
      </div>
      {session.tags.length > 0 ? (
        <div className="flex flex-wrap gap-2">
          {session.tags.map((tag) => (
            <Tag key={tag} label={tag} color="info" />
          ))}
        </div>
      ) : (
        <p className="text-sm text-text-tertiary">
          No memories recorded for this session.
        </p>
      )}
    </div>
  );
}

/* ── Metadata Tab ───────────────────────────────────────────── */
function MetadataTab({ session }: { session: SessionDetail }) {
  const metadataEntries: [string, string | number][] = [
    ['Session ID', session.id],
    ['Status', session.status],
    ['Agent', session.agent],
    ['Project', session.project],
    ['Created', new Date(session.created_at).toLocaleString()],
    ['Last Active', new Date(session.last_active).toLocaleString()],
    ['Duration', `${session.duration_minutes} minutes`],
    ['Turns', session.turn_count],
    ['Memories Created', session.memories_created],
    ['Tokens Used', session.tokens_used],
  ];

  return (
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
          {metadataEntries.map(([key, value]) => (
            <tr
              key={key}
              className="border-b border-border last:border-b-0"
            >
              <td className="px-4 py-3 text-sm font-medium text-text-primary">
                {key}
              </td>
              <td className="px-4 py-3 font-mono text-sm text-text-secondary">
                {String(value)}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

/* ── Page component ─────────────────────────────────────────── */

export function SessionDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data: session, isLoading, isError, error, refetch } = useSession(
    id ?? '',
  );
  const deleteSession = useDeleteSession();
  const resumeSession = useResumeSession();
  const [activeTab, setActiveTab] = useState('timeline');
  const [showDeleteModal, setShowDeleteModal] = useState(false);
  const [showOverflow, setShowOverflow] = useState(false);
  const overflowRef = useRef<HTMLDivElement>(null);

  // Close overflow menu on outside click
  useEffect(() => {
    if (!showOverflow) return;
    const handler = (e: MouseEvent) => {
      if (overflowRef.current && !overflowRef.current.contains(e.target as Node)) {
        setShowOverflow(false);
      }
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [showOverflow]);

  /* ── Loading state ────────────────────────────────────────── */
  if (isLoading) {
    return (
      <div>
        <LoadingSkeleton variant="text" count={1} />
        <div className="mt-lg flex flex-col gap-4">
          <LoadingSkeleton variant="card" count={3} />
        </div>
      </div>
    );
  }

  /* ── Error / not-found state ──────────────────────────────── */
  if (isError || !session) {
    return (
      <div>
        <PageHeader
          title="Session"
          breadcrumbs={[{ label: 'Sessions', href: '/sessions' }]}
        />
        <EmptyState
          icon={Activity}
          title="Session not found"
          message={
            error instanceof Error
              ? error.message
              : 'The requested session could not be found.'
          }
          action={
            <div className="flex gap-3">
              <Link to="/sessions">
                <Button variant="secondary">Back to Sessions</Button>
              </Link>
              <Button
                variant="primary"
                onClick={() => refetch()}
              >
                <RefreshCw className="h-4 w-4" />
                Retry
              </Button>
            </div>
          }
        />
      </div>
    );
  }

  /* ── Delete handler ───────────────────────────────────────── */
  const handleDelete = async () => {
    await deleteSession.mutateAsync(session.id);
    setShowDeleteModal(false);
    navigate('/sessions');
  };

  /* ── Truncated ID for display ─────────────────────────────── */
  const truncatedId =
    session.id.length > 11
      ? `${session.id.slice(0, 11)}…`
      : session.id;

  /* ── Render ──────────────────────────────────────────────── */
  return (
    <div>
      {/* Page header with breadcrumb + delete button */}
      <PageHeader
        title={`Session ${truncatedId}`}
        breadcrumbs={[
          { label: 'Sessions', href: '/sessions' },
          { label: truncatedId },
        ]}
      >
        {/* Resume button — only for active sessions */}
        {session.status === 'active' && (
          <Button
            variant="primary"
            onClick={() => resumeSession.mutate(session.id)}
            loading={resumeSession.isPending}
          >
            <Play className="h-4 w-4" />
            Resume
          </Button>
        )}

        {/* Overflow menu */}
        <div className="relative" ref={overflowRef}>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setShowOverflow(!showOverflow)}
            aria-label="More actions"
            aria-expanded={showOverflow}
          >
            <MoreVertical className="h-4 w-4" />
          </Button>
          {showOverflow && (
            <div className="absolute right-0 z-50 mt-1 min-w-[160px] rounded-lg border border-border bg-surface py-1 shadow-lg">
              <button
                onClick={() => {
                  setShowOverflow(false);
                  setShowDeleteModal(true);
                }}
                className="flex w-full items-center gap-2 px-3 py-2 text-sm text-error transition-colors hover:bg-bg-hover"
              >
                <Trash2 className="h-4 w-4" />
                Delete Session
              </button>
            </div>
          )}
        </div>
      </PageHeader>

      {/* Session info header */}
      <SessionInfoHeader session={session} />

      {/* Tab bar */}
      <div className="mb-lg">
        <TabBar
          tabs={TABS}
          activeTab={activeTab}
          onChange={setActiveTab}
        />
      </div>

      {/* Tab content */}
      {activeTab === 'timeline' && <TurnTimeline turns={session.turns} />}
      {activeTab === 'messages' && <MessagesTab turns={session.turns} />}
      {activeTab === 'memories' && <MemoriesTab session={session} />}
      {activeTab === 'metadata' && <MetadataTab session={session} />}

      {/* Delete confirmation modal */}
      <Modal
        isOpen={showDeleteModal}
        onClose={() => setShowDeleteModal(false)}
        title="Delete Session"
        footer={
          <>
            <Button
              variant="ghost"
              onClick={() => setShowDeleteModal(false)}
            >
              Cancel
            </Button>
            <Button
              variant="primary"
              onClick={handleDelete}
              loading={deleteSession.isPending}
            >
              Delete
            </Button>
          </>
        }
      >
        <p className="text-sm text-text-secondary">
          Are you sure you want to delete this session? This action
          cannot be undone.
        </p>
      </Modal>
    </div>
  );
}
