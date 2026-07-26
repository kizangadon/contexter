import { useQuery } from '@tanstack/react-query';
import type { Skill, SkillDetail } from '@/api/types';
import { api } from '@/api/client';

export function useSkills(filter?: { category?: string }) {
  return useQuery<Skill[]>({
    queryKey: ['skills', filter],
    queryFn: () => api.get<Skill[]>('/skills', filter as Record<string, string | undefined>),
  });
}

export function useSkill(id: string) {
  return useQuery<SkillDetail>({
    queryKey: ['skill', id],
    queryFn: () => api.get<SkillDetail>(`/skills/${id}`),
    enabled: id.length > 0,
  });
}
