import type { Skill, SkillDetail, Session } from '@/api/types';
import { buildSession } from './sessionFactory';

let skillCounter = 0;

export function resetSkillCounters(): void {
  skillCounter = 0;
}

export function buildSkill(overrides?: Partial<Skill>): Skill {
  skillCounter += 1;

  const id = `skl_${String(skillCounter).padStart(6, '0')}`;
  const now = new Date('2026-07-26T00:00:00Z');
  const created = new Date(now.getTime() - 1209600000 * skillCounter);
  const lastUsed = new Date(now.getTime() - 3600000 * (skillCounter - 1));

  const categories = ['code-review', 'debugging', 'refactoring', 'testing', 'architecture'];
  const names = ['Review Pro', 'Bug Hunter', 'Refactor Master', 'Test Sage', 'Architect AI'];

  return {
    id,
    name: names[(skillCounter - 1) % names.length] ?? 'Skill',
    category: categories[(skillCounter - 1) % categories.length] ?? 'general',
    effectiveness_score: 75 + Math.floor(Math.random() * 20),
    usage_count: 10 + Math.floor(Math.random() * 90),
    created_at: created.toISOString(),
    last_used: lastUsed.toISOString(),
    ...overrides,
  };
}

export function buildSkillList(count = 3): Skill[] {
  resetSkillCounters();
  return Array.from({ length: count }, () => buildSkill());
}

export function buildSkillDetail(overrides?: Partial<SkillDetail>): SkillDetail {
  const skill = buildSkill(overrides);
  const sessions: Session[] = [
    buildSession({ status: 'done' }),
    buildSession({ status: 'done' }),
  ];

  return {
    ...skill,
    recent_sessions: sessions,
    effectiveness_history: [
      { date: '2026-07-19', score: 72 },
      { date: '2026-07-20', score: 78 },
      { date: '2026-07-21', score: 80 },
    ],
    ...overrides,
  } as SkillDetail;
}
