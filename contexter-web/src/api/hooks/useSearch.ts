import { useQuery } from '@tanstack/react-query';
import type { SearchResult } from '@/api/types';
import { api } from '@/api/client';

export function useSearch(query: string) {
  return useQuery<SearchResult[]>({
    queryKey: ['search', query],
    queryFn: () => api.get<SearchResult[]>('/search', { q: query }),
    enabled: query.length >= 2,
  });
}
