import type { Memory, MemoryDetail, MemoryVersion } from '@/api/types';

let memoryCounter = 0;
let versionCounter = 0;

export function resetMemoryCounters(): void {
  memoryCounter = 0;
  versionCounter = 0;
}

export function buildMemory(overrides?: Partial<Memory>): Memory {
  memoryCounter += 1;

  const id = `mem_${String(memoryCounter).padStart(6, '0')}`;
  const now = new Date('2026-07-26T00:00:00Z');
  const created = new Date(now.getTime() - 86400000 * memoryCounter);

  return {
    id,
    content: 'A key insight about the system architecture and its scaling properties.',
    memory_type: 'conversation',
    tags: ['architecture', 'insight'],
    source_session: `ses_${String(memoryCounter).padStart(6, '0')}`,
    confidence: 0.85,
    version: 1,
    created_at: created.toISOString(),
    updated_at: created.toISOString(),
    ...overrides,
  };
}

export function buildMemoryList(count = 3): Memory[] {
  resetMemoryCounters();
  return Array.from({ length: count }, () => buildMemory());
}

export function buildMemoryVersion(overrides?: Partial<MemoryVersion>): MemoryVersion {
  versionCounter += 1;

  const now = new Date('2026-07-26T00:00:00Z');
  const created = new Date(now.getTime() - 3600000 * versionCounter);

  return {
    version: versionCounter,
    content: 'Refined understanding of the domain model and bounded contexts.',
    tags: ['architecture', 'domain-driven-design'],
    created_at: created.toISOString(),
    ...overrides,
  };
}

export function buildMemoryDetail(overrides?: Partial<MemoryDetail>): MemoryDetail {
  const memory = buildMemory(overrides);
  return {
    ...memory,
    versions: [buildMemoryVersion({ version: 1 }), buildMemoryVersion({ version: 2 })],
    related_memories: [
      { id: 'mem_000002', content: 'Related insight', similarity: 0.75 },
    ],
    metadata: { source: 'analysis', confidence_level: 'high' },
    ...overrides,
  } as MemoryDetail;
}
