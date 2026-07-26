import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { MemoryRouter } from 'react-router';
import { EntityLink } from './EntityLink';

function renderWithRouter(element: React.ReactElement) {
  return render(<MemoryRouter>{element}</MemoryRouter>);
}

describe('EntityLink', () => {
  it('renders a link with the correct href', () => {
    renderWithRouter(<EntityLink to="/sessions/123">Session 123</EntityLink>);
    const link = screen.getByRole('link', { name: 'Session 123' });
    expect(link).toBeInTheDocument();
    expect(link).toHaveAttribute('href', '/sessions/123');
  });

  it('renders children', () => {
    renderWithRouter(
      <EntityLink to="/memories/456">Memory Content</EntityLink>,
    );
    expect(screen.getByText('Memory Content')).toBeInTheDocument();
  });

  it('renders type indicator dot for session', () => {
    const { container } = renderWithRouter(
      <EntityLink to="/sessions/1" type="session">Session</EntityLink>,
    );
    const dot = container.querySelector('.rounded-full');
    expect(dot).toBeInTheDocument();
  });

  it('renders type indicator dot for memory', () => {
    const { container } = renderWithRouter(
      <EntityLink to="/memories/1" type="memory">Memory</EntityLink>,
    );
    const dot = container.querySelector('.rounded-full');
    expect(dot).toBeInTheDocument();
  });

  it('renders type indicator dot for agent', () => {
    const { container } = renderWithRouter(
      <EntityLink to="/agents/1" type="agent">Agent</EntityLink>,
    );
    const dot = container.querySelector('.rounded-full');
    expect(dot).toBeInTheDocument();
  });

  it('renders type indicator dot for skill', () => {
    const { container } = renderWithRouter(
      <EntityLink to="/skills/1" type="skill">Skill</EntityLink>,
    );
    const dot = container.querySelector('.rounded-full');
    expect(dot).toBeInTheDocument();
  });

  it('does not render type indicator dot when type is not provided', () => {
    const { container } = renderWithRouter(
      <EntityLink to="/test">No Dot</EntityLink>,
    );
    const dots = container.querySelectorAll('.rounded-full');
    expect(dots.length).toBe(0);
  });

  it('applies hover underline class', () => {
    renderWithRouter(<EntityLink to="/test">Hover Test</EntityLink>);
    const link = screen.getByRole('link', { name: 'Hover Test' });
    expect(link.className).toContain('hover:underline');
  });

  it('applies accent text color', () => {
    renderWithRouter(<EntityLink to="/test">Accent Test</EntityLink>);
    const link = screen.getByRole('link', { name: 'Accent Test' });
    expect(link.className).toContain('text-accent');
  });

  it('applies custom className', () => {
    renderWithRouter(
      <EntityLink to="/test" className="custom-link">Custom</EntityLink>,
    );
    const link = screen.getByRole('link', { name: 'Custom' });
    expect(link.className).toContain('custom-link');
  });
});
