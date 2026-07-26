import { createPortal } from 'react-dom';
import { AnimatePresence } from 'framer-motion';
import { Toast, type ToastVariant } from './Toast';

/** A toast entry used by ToastContainer */
export interface ToastData {
  id: string;
  message: string;
  variant?: ToastVariant;
  duration?: number;
}

export interface ToastContainerProps {
  /** Current list of active toasts */
  toasts: ToastData[];
  /** Called when a toast should be dismissed */
  onClose: (id: string) => void;
}

export function ToastContainer({ toasts, onClose }: ToastContainerProps) {
  if (toasts.length === 0) return null;

  return createPortal(
    <div
      className="fixed right-4 top-4 z-[60] flex w-80 flex-col gap-2"
      data-testid="toast-container"
    >
      <AnimatePresence>
        {toasts.map((toast) => (
          <Toast
            key={toast.id}
            id={toast.id}
            message={toast.message}
            variant={toast.variant}
            duration={toast.duration}
            onClose={onClose}
          />
        ))}
      </AnimatePresence>
    </div>,
    document.body,
  );
}
