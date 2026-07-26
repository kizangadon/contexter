import { render, screen, act, cleanup } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi, afterEach } from 'vitest';
import { ToastProvider } from './ToastProvider';

// Mock framer-motion for jsdom tests — renders as plain HTML elements
import React from 'react';

const MOTION_FILTER_PROPS = [
  'animate', 'initial', 'exit', 'transition', 'variants',
  'whileHover', 'whileTap', 'whileFocus', 'whileDrag', 'whileInView',
  'onAnimationComplete', 'layout', 'layoutId', 'layoutDependency',
];

vi.mock('framer-motion', () => ({
  motion: new Proxy(
    {},
    {
      get: (_target, prop: string) => {
        const tag = ['div', 'button', 'span', 'section', 'nav', 'header', 'footer', 'main', 'aside', 'ul', 'ol', 'li', 'p', 'h1', 'h2', 'h3', 'h4'].includes(prop) ? prop : 'div';
        const Component = React.forwardRef<HTMLElement, Record<string, unknown>>(
          (props, ref) => {
            const filtered = Object.fromEntries(
              Object.entries(props).filter(
                ([key]) => !MOTION_FILTER_PROPS.includes(key),
              ),
            );
            return React.createElement(tag, { ...filtered, ref });
          },
        );
        Component.displayName = `motion.${tag}`;
        return Component;
      },
    },
  ),
  AnimatePresence: ({ children }: { children: React.ReactNode }) => <>{children}</>,
  motionValue: (v: unknown) => ({ get: () => v, set: () => {}, onChange: () => {} }),
}));

// Helper to dispatch an api:error event
function dispatchApiError(message: string, status: number) {
  window.dispatchEvent(new CustomEvent('api:error', {
    detail: { message, status },
  }));
}

describe('ToastProvider', () => {
  // Explicit cleanup prevents portal-removal race with global afterEach
  afterEach(() => {
    cleanup();
    document.querySelectorAll('[data-testid="toast-container"]').forEach(el => el.remove());
  });

  it('renders children', () => {
    render(
      <ToastProvider>
        <div data-testid="child">Hello</div>
      </ToastProvider>,
    );
    expect(screen.getByTestId('child')).toHaveTextContent('Hello');
  });

  it('shows error toast after api:error event dispatch', () => {
    render(
      <ToastProvider>
        <div>Content</div>
      </ToastProvider>,
    );

    act(() => {
      dispatchApiError('Internal server error', 500);
    });

    expect(screen.getByText('Internal server error')).toBeInTheDocument();
  });

  it('uses warning variant for 4xx errors', () => {
    render(
      <ToastProvider>
        <div>Content</div>
      </ToastProvider>,
    );

    act(() => {
      dispatchApiError('Not found', 404);
    });

    expect(screen.getByText('Not found')).toBeInTheDocument();
  });

  it('accumulates multiple toasts from separate events', () => {
    render(
      <ToastProvider>
        <div>Content</div>
      </ToastProvider>,
    );

    act(() => {
      dispatchApiError('First error', 500);
    });
    act(() => {
      dispatchApiError('Second error', 400);
    });

    expect(screen.getByText('First error')).toBeInTheDocument();
    expect(screen.getByText('Second error')).toBeInTheDocument();
  });

  it('dismisses toast when close button is clicked', async () => {
    const user = userEvent.setup();

    render(
      <ToastProvider>
        <div>Content</div>
      </ToastProvider>,
    );

    act(() => {
      dispatchApiError('Dismiss me', 500);
    });

    expect(screen.getByText('Dismiss me')).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: /close/i }));

    expect(screen.queryByText('Dismiss me')).not.toBeInTheDocument();
  });
});
