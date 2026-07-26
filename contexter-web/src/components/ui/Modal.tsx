import {
  type ReactNode,
  useCallback,
  useEffect,
  useRef,
} from 'react';
import { createPortal } from 'react-dom';
import { AnimatePresence, motion } from 'framer-motion';
import { X } from 'lucide-react';

export interface ModalProps {
  /** Whether the modal is currently visible */
  isOpen: boolean;
  /** Called when the modal should close (Esc, backdrop click, close button) */
  onClose: () => void;
  /** Optional heading rendered in the modal header */
  title?: string;
  /** Modal body content */
  children: ReactNode;
  /** Optional content rendered in the modal footer (e.g. action buttons) */
  footer?: ReactNode;
}

/* ── Animation variants ────────────────────────────────────── */
const overlayVariants = {
  hidden: { opacity: 0 },
  visible: { opacity: 1 },
};

const surfaceVariants = {
  hidden: { opacity: 0, scale: 0.95 },
  visible: { opacity: 1, scale: 1 },
};

/* ── Selector for focusable elements ───────────────────────── */
const FOCUSABLE_SELECTOR =
  'a[href], button:not([disabled]), textarea:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])';

export function Modal({
  isOpen,
  onClose,
  title,
  children,
  footer,
}: ModalProps) {
  const surfaceRef = useRef<HTMLDivElement>(null);

  /* ── Focus trap + Esc handler ──────────────────────────── */
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
        return;
      }

      if (e.key === 'Tab') {
        const surface = surfaceRef.current;
        if (!surface) return;

        const focusable = surface.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR);
        if (focusable.length === 0) return;

        const first = focusable[0]!;
        const last = focusable[focusable.length - 1]!;

        if (e.shiftKey) {
          if (document.activeElement === first) {
            e.preventDefault();
            last.focus();
          }
        } else {
          if (document.activeElement === last) {
            e.preventDefault();
            first.focus();
          }
        }
      }
    },
    [onClose],
  );

  /* ── Set up focus trap when modal opens ─────────────────── */
  useEffect(() => {
    if (!isOpen) return;

    const surface = surfaceRef.current;
    if (!surface) return;

    // Focus the first focusable element inside the modal
    const focusable = surface.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR);
    if (focusable.length > 0) {
      focusable[0]!.focus();
    }

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, handleKeyDown]);

  /* ── Prevent body scroll while open ─────────────────────── */
  useEffect(() => {
    if (isOpen) {
      const prev = document.body.style.overflow;
      document.body.style.overflow = 'hidden';
      return () => {
        document.body.style.overflow = prev;
      };
    }
  }, [isOpen]);

  return createPortal(
    <AnimatePresence>
      {isOpen && (
        <motion.div
          data-testid="modal-overlay"
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60"
          variants={overlayVariants}
          initial="hidden"
          animate="visible"
          exit="hidden"
          transition={{ duration: 0.2 }}
          onClick={(e) => {
            // Only close when clicking the overlay itself, not the surface
            if (e.target === e.currentTarget) {
              onClose();
            }
          }}
        >
          <motion.div
            ref={surfaceRef}
            role="dialog"
            aria-modal="true"
            aria-label={title ?? 'Modal dialog'}
            className="flex max-h-[85vh] w-full max-w-lg flex-col rounded-lg bg-surface shadow-xl"
            variants={surfaceVariants}
            initial="hidden"
            animate="visible"
            exit="hidden"
            transition={{ duration: 0.2 }}
            onClick={(e) => e.stopPropagation()}
          >
            {/* ── Header (always rendered — contains close button) ── */}
            <div className="flex items-center justify-between border-b border-border px-6 py-4">
              {title && (
                <h2 className="text-lg font-semibold text-text-primary">
                  {title}
                </h2>
              )}
              <button
                type="button"
                onClick={onClose}
                className="ml-auto rounded-md p-1 text-text-tertiary transition-colors hover:text-text-primary hover:bg-bg-hover"
                aria-label="Close"
              >
                <X className="h-5 w-5" />
              </button>
            </div>

            {/* ── Body ────────────────────────────────────── */}
            <div className="overflow-y-auto px-6 py-4">{children}</div>

            {/* ── Footer ──────────────────────────────────── */}
            {footer && (
              <div className="flex items-center justify-end gap-3 border-t border-border px-6 py-4">
                {footer}
              </div>
            )}
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>,
    document.body,
  );
}
