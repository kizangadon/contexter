import { useEffect } from 'react';
import { motion } from 'framer-motion';
import { X } from 'lucide-react';

export type ToastVariant = 'success' | 'error' | 'info' | 'warning';

export interface ToastProps {
  /** Unique identifier for this toast */
  id: string;
  /** Message text displayed in the toast */
  message: string;
  /** Visual variant — maps to V2-DEEP semantic tokens */
  variant?: ToastVariant;
  /** Called when the toast should be dismissed (auto or manual) */
  onClose: (id: string) => void;
  /** Auto-dismiss duration in ms (default: 4000) */
  duration?: number;
}

/* ── Variant → styles ──────────────────────────────────────── */
const variantStyles: Record<ToastVariant, string> = {
  success: 'border-l-success bg-success/10',
  error: 'border-l-error bg-error/10',
  info: 'border-l-info bg-info/10',
  warning: 'border-l-warning bg-warning/10',
};

const iconColors: Record<ToastVariant, string> = {
  success: 'text-success',
  error: 'text-error',
  info: 'text-info',
  warning: 'text-warning',
};

/* ── Slide-in from right animation ─────────────────────────── */
const toastVariants = {
  initial: { opacity: 0, x: 80 },
  animate: { opacity: 1, x: 0 },
  exit: { opacity: 0, x: 80 },
};

export function Toast({
  id,
  message,
  variant = 'info',
  onClose,
  duration = 4000,
}: ToastProps) {
  /* ── Auto-dismiss timer ────────────────────────────────── */
  useEffect(() => {
    if (duration <= 0) return;

    const timer = setTimeout(() => {
      onClose(id);
    }, duration);

    return () => clearTimeout(timer);
  }, [id, duration, onClose]);

  return (
    <motion.div
      layout
      variants={toastVariants}
      initial="initial"
      animate="animate"
      exit="exit"
      transition={{ type: 'spring', stiffness: 400, damping: 30 }}
      className={`flex items-start gap-3 rounded-md border-l-4 p-4 shadow-lg ${variantStyles[variant]}`}
      role="status"
      aria-live="polite"
    >
      {/* ── Message ────────────────────────────────────────── */}
      <p className={`flex-1 text-sm ${iconColors[variant]}`}>{message}</p>

      {/* ── Close button ───────────────────────────────────── */}
      <button
        type="button"
        onClick={() => onClose(id)}
        className="rounded p-0.5 text-text-tertiary transition-colors hover:text-text-primary"
        aria-label="Close"
      >
        <X className="h-4 w-4" />
      </button>
    </motion.div>
  );
}
