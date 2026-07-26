import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { AgentCard } from './AgentCard';
import type { Agent } from '@/api/types';

const mockAgent: Agent = {
  id: 'agt_000001',
  name: 'Code-Reviewer',
  capabilities: ['code-review', 'debugging', 'refactoring'],
  status: 'active',
  efficiency_score: 85,
  sessions_count: 42,
  avg_latency_ms: 320,
  created_at: '2026-07-01T00:00:00Z',
  last_active: '2026-07-26T00:00:00Z',
};

describe('AgentCard', () => {
  it('renders agent name', () => {
    render(<AgentCard agent={mockAgent} onClick={() => {}} />);
    expect(screen.getByText('Code-Reviewer')).toBeInTheDocument();
  });

  it('renders status badge with agent status', () => {
    render(<AgentCard agent={mockAgent} onClick={() => {}} />);
    expect(screen.getByText('active')).toBeInTheDocument();
  });

  it('renders all capability tags', () => {
    render(<AgentCard agent={mockAgent} onClick={() => {}} />);
    for (const cap of mockAgent.capabilities) {
      expect(screen.getByText(cap)).toBeInTheDocument();
    }
  });

  it('renders efficiency score', () => {
    render(<AgentCard agent={mockAgent} onClick={() => {}} />);
    expect(screen.getByText('85%')).toBeInTheDocument();
  });

  it('renders sessions count', () => {
    render(<AgentCard agent={mockAgent} onClick={() => {}} />);
    expect(screen.getByText('42 sessions')).toBeInTheDocument();
  });

  it('calls onClick when card is clicked', async () => {
    const onClick = vi.fn();
    const user = userEvent.setup();
    render(<AgentCard agent={mockAgent} onClick={onClick} />);

    await user.click(screen.getByRole('button'));
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it('renders card as a button role', () => {
    render(<AgentCard agent={mockAgent} onClick={() => {}} />);
    expect(screen.getByRole('button')).toBeInTheDocument();
  });

  it('renders efficiency bar with correct width', () => {
    const { container } = render(
      <AgentCard agent={mockAgent} onClick={() => {}} />,
    );
    const bar = container.querySelector('[data-testid="efficiency-bar"]');
    expect(bar).toBeInTheDocument();
    expect(bar).toHaveStyle({ width: '85%' });
  });

  it('handles offline status', () => {
    const offlineAgent: Agent = { ...mockAgent, status: 'offline', efficiency_score: 0 };
    render(<AgentCard agent={offlineAgent} onClick={() => {}} />);
    expect(screen.getByText('offline')).toBeInTheDocument();
    expect(screen.getByText('0%')).toBeInTheDocument();
  });

  it('handles error status', () => {
    const errorAgent: Agent = { ...mockAgent, status: 'error' };
    render(<AgentCard agent={errorAgent} onClick={() => {}} />);
    expect(screen.getByText('error')).toBeInTheDocument();
  });
});
