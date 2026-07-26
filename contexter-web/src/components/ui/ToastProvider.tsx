import { useState, useEffect, useCallback, type ReactNode } from 'react';
import { ToastContainer, type ToastData } from './ToastContainer';

/** Counter for generating unique toast IDs */
let toastIdCounter = 0;

export interface ToastProviderProps {
  children: ReactNode;
}

/**
 * Listens for `api:error` custom events on `window` and displays
 * corresponding toast notifications via ToastContainer.
 */
export function ToastProvider({ children }: ToastProviderProps) {
  const [toasts, setToasts] = useState<ToastData[]>([]);

  const removeToast = useCallback((id: string) => {
    setToasts(prev => prev.filter(t => t.id !== id));
  }, []);

  // Intentionally empty deps: this effect runs once on mount to register
  // the global `api:error` listener and cleans it up on unmount. The handler
  // is stable (no external references) so it never needs to re-run.
  useEffect(() => {
    const handler = (event: Event) => {
      const detail = (event as CustomEvent<{ message: string; status: number }>).detail;
      const id = `api-error-${++toastIdCounter}`;
      const variant = detail.status >= 500 ? 'error' : 'warning';

      setToasts(prev => [...prev, {
        id,
        message: detail.message,
        variant,
        duration: 6000,
      }]);
    };

    window.addEventListener('api:error', handler);
    return () => window.removeEventListener('api:error', handler);
  }, []);

  return (
    <>
      {children}
      <ToastContainer toasts={toasts} onClose={removeToast} />
    </>
  );
}
