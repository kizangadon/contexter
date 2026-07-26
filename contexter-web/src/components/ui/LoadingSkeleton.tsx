export type SkeletonVariant =
  | 'text'
  | 'card'
  | 'table-row'
  | 'avatar';

export interface LoadingSkeletonProps {
  /** Shape variant */
  variant?: SkeletonVariant;
  /** Number of skeleton items to render (default: 1) */
  count?: number;
  /** Additional CSS class names */
  className?: string;
}

/* ── Variant → style presets ───────────────────────────────── */
const variantStyles: Record<SkeletonVariant, string> = {
  text: 'h-4 w-full rounded',
  card: 'h-32 w-full rounded-lg',
  'table-row': 'h-10 w-full rounded',
  avatar: 'h-10 w-10 rounded-full',
};

/* ── Inline keyframes (injected once) ──────────────────────── */
const pulseKeyframes = `
@keyframes skeleton-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}
`;

let keyframesInjected = false;

function ensureKeyframes() {
  if (keyframesInjected) return;
  const style = document.createElement('style');
  style.textContent = pulseKeyframes;
  document.head.appendChild(style);
  keyframesInjected = true;
}

export function LoadingSkeleton({
  variant = 'text',
  count = 1,
  className = '',
}: LoadingSkeletonProps) {
  // Inject keyframes on first render
  if (typeof document !== 'undefined') {
    ensureKeyframes();
  }

  const baseStyle: React.CSSProperties = {
    animation: 'skeleton-pulse 2s ease-in-out infinite',
  };

  return (
    <>
      {Array.from({ length: count }, (_, i) => (
        <div
          key={i}
          data-testid="skeleton"
          className={`bg-bg-tertiary ${variantStyles[variant]} ${className}`}
          style={baseStyle}
          aria-hidden="true"
        />
      ))}
    </>
  );
}
