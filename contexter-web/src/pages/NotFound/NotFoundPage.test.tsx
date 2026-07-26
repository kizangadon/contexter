import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { MemoryRouter } from 'react-router';
import type { ReactNode } from 'react';
import { NotFoundPage } from './NotFoundPage';

function createWrapper() {
  return function Wrapper({ children }: { children: ReactNode }) {
    return <MemoryRouter initialEntries={['/nonexistent']}>{children}</MemoryRouter>;
  };
}

describe('NotFoundPage', () => {
  it('renders a 404 heading', () => {
    render(<NotFoundPage />, { wrapper: createWrapper() });
    expect(screen.getByText(/404/i)).toBeInTheDocument();
  });

  it('renders "Page not found" message', () => {
    render(<NotFoundPage />, { wrapper: createWrapper() });
    expect(screen.getByText(/page not found/i)).toBeInTheDocument();
  });

  it('renders a link back to the dashboard', () => {
    render(<NotFoundPage />, { wrapper: createWrapper() });
    const link = screen.getByRole('link', { name: /back to dashboard/i });
    expect(link).toBeInTheDocument();
    expect(link).toHaveAttribute('href', '/dashboard');
  });
});
