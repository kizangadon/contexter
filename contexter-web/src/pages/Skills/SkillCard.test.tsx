import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { SkillCard } from './SkillCard';
import type { Skill } from '@/api/types';

function buildSkill(overrides?: Partial<Skill>): Skill {
  return {
    id: 'skl_000001',
    name: 'Review Pro',
    category: 'code-review',
    effectiveness_score: 85,
    usage_count: 42,
    created_at: '2026-07-20T00:00:00Z',
    last_used: '2026-07-25T00:00:00Z',
    ...overrides,
  };
}

describe('SkillCard', () => {
  it('renders skill name', () => {
    const skill = buildSkill();
    render(<SkillCard skill={skill} onClick={() => {}} />);

    expect(screen.getByText('Review Pro')).toBeInTheDocument();
  });

  it('renders category badge', () => {
    const skill = buildSkill({ category: 'debugging' });
    render(<SkillCard skill={skill} onClick={() => {}} />);

    expect(screen.getByText('debugging')).toBeInTheDocument();
  });

  it('renders effectiveness score as a bar', () => {
    const skill = buildSkill({ effectiveness_score: 75 });
    render(<SkillCard skill={skill} onClick={() => {}} />);

    expect(screen.getByText('75%')).toBeInTheDocument();
    // The bar element should reflect the width
    const barFill = screen.getByTestId('effectiveness-bar-fill');
    expect(barFill).toBeInTheDocument();
    expect(barFill).toHaveStyle({ width: '75%' });
  });

  it('renders usage count', () => {
    const skill = buildSkill({ usage_count: 99 });
    render(<SkillCard skill={skill} onClick={() => {}} />);

    expect(screen.getByText('99 uses')).toBeInTheDocument();
  });

  it('calls onClick when clicked', async () => {
    const onClick = vi.fn();
    const skill = buildSkill({ id: 'skl_000001' });
    render(<SkillCard skill={skill} onClick={onClick} />);

    await userEvent.click(screen.getByTestId('skill-card'));

    expect(onClick).toHaveBeenCalledTimes(1);
    expect(onClick).toHaveBeenCalledWith('skl_000001');
  });

  it('renders singular "use" when usage_count is 1', () => {
    const skill = buildSkill({ usage_count: 1 });
    render(<SkillCard skill={skill} onClick={() => {}} />);

    expect(screen.getByText('1 use')).toBeInTheDocument();
  });
});
