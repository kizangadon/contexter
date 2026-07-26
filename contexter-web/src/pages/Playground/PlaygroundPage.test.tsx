import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { MemoryRouter } from 'react-router';
import type { ReactNode } from 'react';
import { PlaygroundPage } from './PlaygroundPage';

function createWrapper() {
  return function Wrapper({ children }: { children: ReactNode }) {
    return <MemoryRouter initialEntries={['/playground']}>{children}</MemoryRouter>;
  };
}

describe('PlaygroundPage', () => {
  it('renders the page header title', () => {
    render(<PlaygroundPage />, { wrapper: createWrapper() });
    expect(screen.getByRole('heading', { name: 'Playground' })).toBeInTheDocument();
  });

  it('renders a textarea for input', () => {
    render(<PlaygroundPage />, { wrapper: createWrapper() });
    expect(screen.getByRole('textbox')).toBeInTheDocument();
  });

  it('renders a submit button', () => {
    render(<PlaygroundPage />, { wrapper: createWrapper() });
    expect(screen.getByRole('button', { name: /submit|send|run/i })).toBeInTheDocument();
  });

  it('renders a response display area', () => {
    render(<PlaygroundPage />, { wrapper: createWrapper() });
    expect(screen.getByRole('heading', { name: /response/i })).toBeInTheDocument();
  });
});
