import { http, HttpResponse } from 'msw';
import type { HttpHandler } from 'msw';
import type { SkillDetail } from '@/api/types';
import { buildSkillDetail } from '../factories/skillFactory';

const skills = new Map<string, SkillDetail>();

function seedSkills(): void {
  const ids = ['skl_000001', 'skl_000002', 'skl_000003'];
  for (const id of ids) {
    const detail = buildSkillDetail({ id });
    skills.set(id, detail);
  }
}
seedSkills();

export const skillsHandlers: HttpHandler[] = [
  // GET /api/v1/skills — list with optional category filter
  http.get('*/api/v1/skills', ({ request }) => {
    const url = new URL(request.url);
    const category = url.searchParams.get('category');

    let list = Array.from(skills.values());
    if (category) {
      list = list.filter((s) => s.category === category);
    }
    return HttpResponse.json(list);
  }),

  // GET /api/v1/skills/:id — skill detail
  http.get('*/api/v1/skills/:id', ({ params }) => {
    const skill = skills.get(params.id as string);
    if (!skill) {
      return HttpResponse.json({ detail: 'Skill not found' }, { status: 404 });
    }
    return HttpResponse.json(skill);
  }),
];
