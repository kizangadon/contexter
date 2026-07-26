import { useQuery } from '@tanstack/react-query';
import type { AuditEntry } from '@/api/types';
import { api } from '@/api/client';

export function useAudit() {
  return useQuery<AuditEntry[]>({
    queryKey: ['audit'],
    queryFn: () => api.get<AuditEntry[]>('/audit'),
  });
}
