import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
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

import { Modal } from './Modal';

describe('Modal', () => {
  it('renders content when isOpen is true', () => {
    render(
      <Modal isOpen={true} onClose={vi.fn()}>
        <p>Modal content</p>
      </Modal>,
    );
    expect(screen.getByText('Modal content')).toBeInTheDocument();
  });

  it('does not render content when isOpen is false', () => {
    render(
      <Modal isOpen={false} onClose={vi.fn()}>
        <p>Modal content</p>
      </Modal>,
    );
    expect(screen.queryByText('Modal content')).not.toBeInTheDocument();
  });

  it('renders title when provided', () => {
    render(
      <Modal isOpen={true} onClose={vi.fn()} title="My Title">
        <p>Content</p>
      </Modal>,
    );
    expect(screen.getByText('My Title')).toBeInTheDocument();
  });

  it('renders footer when provided', () => {
    render(
      <Modal isOpen={true} onClose={vi.fn()} footer={<button>Save</button>}>
        <p>Content</p>
      </Modal>,
    );
    expect(screen.getByRole('button', { name: 'Save' })).toBeInTheDocument();
  });

  it('fires onClose when Escape key is pressed', async () => {
    const handleClose = vi.fn();
    const user = userEvent.setup();

    render(
      <Modal isOpen={true} onClose={handleClose}>
        <p>Content</p>
      </Modal>,
    );

    await user.keyboard('{Escape}');
    expect(handleClose).toHaveBeenCalledTimes(1);
  });

  it('fires onClose when backdrop is clicked', async () => {
    const handleClose = vi.fn();
    const user = userEvent.setup();

    render(
      <Modal isOpen={true} onClose={handleClose}>
        <p>Content</p>
      </Modal>,
    );

    // Click the overlay backdrop (the outermost portal div)
    const overlay = screen.getByTestId('modal-overlay');
    await user.click(overlay);
    expect(handleClose).toHaveBeenCalledTimes(1);
  });

  it('does not fire onClose when content area is clicked', async () => {
    const handleClose = vi.fn();
    const user = userEvent.setup();

    render(
      <Modal isOpen={true} onClose={handleClose}>
        <p>Content</p>
      </Modal>,
    );

    const content = screen.getByText('Content');
    await user.click(content);
    expect(handleClose).not.toHaveBeenCalled();
  });

  it('renders close button', () => {
    render(
      <Modal isOpen={true} onClose={vi.fn()}>
        <p>Content</p>
      </Modal>,
    );
    expect(screen.getByRole('button', { name: /close/i })).toBeInTheDocument();
  });
});
