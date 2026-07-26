import { act, render, screen } from '@testing-library/react';
import { describe, expect, it, vi, afterEach } from 'vitest';
import { Toast } from './Toast';

describe('Toast', () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it('renders toast with message', () => {
    render(<Toast id="1" message="Task saved" onClose={vi.fn()} />);
    expect(screen.getByText('Task saved')).toBeInTheDocument();
  });

  it.each(['success', 'error', 'info', 'warning'] as const)(
    'applies correct styling for %s variant',
    (variant) => {
      render(
        <Toast id="1" message={variant} variant={variant} onClose={vi.fn()} />,
      );
      const toast = screen.getByText(variant);
      expect(toast.className).toContain(variant);
    },
  );

  it('fires onClose after default duration', () => {
    vi.useFakeTimers();
    const handleClose = vi.fn();

    render(<Toast id="1" message="Auto dismiss" onClose={handleClose} />);

    act(() => {
      vi.advanceTimersByTime(4000);
    });

    expect(handleClose).toHaveBeenCalledWith('1');
  });

  it('fires onClose after custom duration', () => {
    vi.useFakeTimers();
    const handleClose = vi.fn();

    render(
      <Toast id="1" message="Custom duration" onClose={handleClose} duration={2000} />,
    );

    act(() => {
      vi.advanceTimersByTime(2000);
    });

    expect(handleClose).toHaveBeenCalledWith('1');
  });

  it('does not fire onClose before duration elapses', () => {
    vi.useFakeTimers();
    const handleClose = vi.fn();

    render(<Toast id="1" message="Not yet" onClose={handleClose} />);

    act(() => {
      vi.advanceTimersByTime(3000);
    });

    expect(handleClose).not.toHaveBeenCalled();
  });

  it('renders close button', () => {
    render(<Toast id="1" message="Dismiss me" onClose={vi.fn()} />);
    expect(screen.getByRole('button', { name: /close/i })).toBeInTheDocument();
  });

  it('fires onClose when close button is clicked', async () => {
    const userEventModule = await import('@testing-library/user-event');
    const user = userEventModule.default.setup();
    const handleClose = vi.fn();

    render(<Toast id="1" message="Dismiss" onClose={handleClose} />);

    await user.click(screen.getByRole('button', { name: /close/i }));
    expect(handleClose).toHaveBeenCalledWith('1');
  });
});
