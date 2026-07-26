import { http, HttpResponse } from 'msw';
import type { HttpHandler } from 'msw';
import type { SettingsSection } from '@/api/types';

const settingsStore = new Map<string, SettingsSection>([
  [
    'general',
    {
      key: 'general',
      label: 'General Settings',
      settings: { theme: 'dark', language: 'en', notifications_enabled: true },
    },
  ],
  [
    'providers',
    {
      key: 'providers',
      label: 'AI Providers',
      settings: {
        providers: [
          { name: 'OpenAI', type: 'openai', enabled: true, config: { model: 'gpt-4', max_tokens: '4096' } },
          { name: 'Anthropic', type: 'anthropic', enabled: true, config: { model: 'claude-3', max_tokens: '4096' } },
        ],
      },
    },
  ],
]);

export const settingsHandlers: HttpHandler[] = [
  // GET /api/v1/settings/:section
  http.get('*/api/v1/settings/:section', ({ params }) => {
    const section = settingsStore.get(params.section as string);
    if (!section) {
      return HttpResponse.json({ detail: 'Section not found' }, { status: 404 });
    }
    return HttpResponse.json(section);
  }),

  // PUT /api/v1/settings/:section — update entire section
  http.put('*/api/v1/settings/:section', async ({ params, request }) => {
    const body = (await request.json()) as Record<string, unknown>;
    const existing = settingsStore.get(params.section as string);
    if (!existing) {
      return HttpResponse.json({ detail: 'Section not found' }, { status: 404 });
    }
    const settingsPayload = (body.settings ?? body) as Record<string, unknown>;
    const updated: SettingsSection = {
      ...existing,
      settings: settingsPayload,
    };
    settingsStore.set(params.section as string, updated);
    return HttpResponse.json(updated);
  }),
];
