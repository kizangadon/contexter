import { http, HttpResponse } from 'msw';
import type { HttpHandler } from 'msw';
import type { Memory, MemoryDetail } from '@/api/types';
import { buildMemoryDetail, buildMemoryVersion } from '../factories/memoryFactory';

const memories = new Map<string, MemoryDetail>();

function seedMemories(): void {
  const ids = ['mem_000001', 'mem_000002', 'mem_000003'];
  for (const id of ids) {
    const detail = buildMemoryDetail({ id });
    memories.set(id, detail);
  }
}
seedMemories();

export const memoriesHandlers: HttpHandler[] = [
  // GET /api/v1/memories — list with optional type/tags filter
  http.get('*/api/v1/memories', ({ request }) => {
    const url = new URL(request.url);
    const memoryType = url.searchParams.get('memory_type');
    const tagsParam = url.searchParams.get('tags');

    let list = Array.from(memories.values());

    if (memoryType) {
      list = list.filter((m) => m.memory_type === memoryType);
    }
    if (tagsParam) {
      const tags = tagsParam.split(',');
      list = list.filter((m) => tags.some((t) => m.tags.includes(t)));
    }

    return HttpResponse.json(list);
  }),

  // GET /api/v1/memories/:id — memory detail
  http.get('*/api/v1/memories/:id', ({ params }) => {
    const memory = memories.get(params.id as string);
    if (!memory) {
      return HttpResponse.json({ detail: 'Memory not found' }, { status: 404 });
    }
    return HttpResponse.json(memory);
  }),

  // GET /api/v1/memories/:id/versions — list memory versions
  http.get('*/api/v1/memories/:id/versions', ({ params }) => {
    const memory = memories.get(params.id as string);
    if (!memory) {
      return HttpResponse.json({ detail: 'Memory not found' }, { status: 404 });
    }
    return HttpResponse.json(memory.versions);
  }),

  // GET /api/v1/memories/search — search memories
  http.get('*/api/v1/memories/search', ({ request }) => {
    const url = new URL(request.url);
    const query = url.searchParams.get('q')?.toLowerCase() ?? '';

    const results = Array.from(memories.values()).filter(
      (m) => m.content.toLowerCase().includes(query) || m.tags.some((t) => t.includes(query)),
    );

    return HttpResponse.json(results);
  }),

  // POST /api/v1/memories — create
  http.post('*/api/v1/memories', async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown> | undefined;
    const memory = buildMemoryDetail({
      content: (body?.content as string) ?? undefined,
      memory_type: (body?.memory_type as Memory['memory_type']) ?? 'conversation',
    });
    memories.set(memory.id, memory);
    return HttpResponse.json(memory, { status: 201 });
  }),

  // PATCH /api/v1/memories/:id — update
  http.patch('*/api/v1/memories/:id', async ({ params, request }) => {
    const existing = memories.get(params.id as string);
    if (!existing) {
      return HttpResponse.json({ detail: 'Memory not found' }, { status: 404 });
    }
    const body = (await request.json()) as Record<string, unknown>;
    const newVersion = buildMemoryVersion({
      version: existing.version + 1,
      content: existing.content,
      tags: existing.tags,
    });
    const updated: MemoryDetail = {
      ...existing,
      ...body,
      version: existing.version + 1,
      versions: [...existing.versions, newVersion],
      updated_at: new Date().toISOString(),
    };
    memories.set(params.id as string, updated);
    return HttpResponse.json(updated);
  }),

  // DELETE /api/v1/memories/:id — delete
  http.delete('*/api/v1/memories/:id', ({ params }) => {
    if (!memories.has(params.id as string)) {
      return HttpResponse.json({ detail: 'Memory not found' }, { status: 404 });
    }
    memories.delete(params.id as string);
    return HttpResponse.json(null, { status: 204 });
  }),
];
