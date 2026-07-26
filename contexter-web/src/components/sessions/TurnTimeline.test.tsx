import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { Turn } from '@/api/types';
import { TurnTimeline } from './TurnTimeline';

function buildMockTurn(overrides?: Partial<Turn>): Turn {
  return {
    id: 'trn_000001',
    session_id: 'ses_000001',
    number: 1,
    role: 'user',
    content: 'Hello, agent!',
    latency_ms: 100,
    created_at: new Date('2026-07-26T00:00:00Z').toISOString(),
    ...overrides,
  };
}

describe('TurnTimeline', () => {
  it('renders a list of turns as message bubbles', () => {
    const turns: Turn[] = [
      buildMockTurn({ id: 'trn_001', content: 'First turn', role: 'user' }),
      buildMockTurn({ id: 'trn_002', content: 'Second turn', role: 'agent' }),
    ];

    render(<TurnTimeline turns={turns} />);

    expect(screen.getByText('First turn')).toBeInTheDocument();
    expect(screen.getByText('Second turn')).toBeInTheDocument();
  });

  it('renders the correct number of turns', () => {
    const turns: Turn[] = [
      buildMockTurn({ id: 'trn_001' }),
      buildMockTurn({ id: 'trn_002' }),
      buildMockTurn({ id: 'trn_003' }),
    ];

    render(<TurnTimeline turns={turns} />);

    const turnLabels = screen.getAllByText(/^Turn \d+$/);
    expect(turnLabels).toHaveLength(3);
  });

  it('shows turn numbers for each turn', () => {
    const turns: Turn[] = [
      buildMockTurn({ id: 'trn_001', role: 'user' }),
      buildMockTurn({ id: 'trn_002', role: 'agent' }),
    ];

    render(<TurnTimeline turns={turns} />);

    expect(screen.getByText('Turn 1')).toBeInTheDocument();
    expect(screen.getByText('Turn 2')).toBeInTheDocument();
  });

  it('shows empty state message when turns array is empty', () => {
    render(<TurnTimeline turns={[]} />);

    expect(
      screen.getByText('No turns recorded in this session.'),
    ).toBeInTheDocument();
  });

  it('does not render any message content when turns is empty', () => {
    render(<TurnTimeline turns={[]} />);

    expect(screen.queryByText('Hello, agent!')).not.toBeInTheDocument();
  });
});
