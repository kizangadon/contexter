import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

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

import { ToastContainer } from './ToastContainer';

describe('ToastContainer', () => {
  it('renders multiple toasts stacked', () => {
    const toasts = [
      { id: '1', message: 'First toast', variant: 'success' as const },
      { id: '2', message: 'Second toast', variant: 'error' as const },
    ];

    render(<ToastContainer toasts={toasts} onClose={vi.fn()} />);

    expect(screen.getByText('First toast')).toBeInTheDocument();
    expect(screen.getByText('Second toast')).toBeInTheDocument();
  });

  it('renders nothing when toast list is empty', () => {
    const { container } = render(
      <ToastContainer toasts={[]} onClose={vi.fn()} />,
    );

    // Portal should not contain any toast elements
    const portalContent = container.querySelector('[data-testid="toast-container"]');
    expect(portalContent).not.toBeInTheDocument();
  });
});
