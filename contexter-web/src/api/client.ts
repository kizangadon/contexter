const BASE_URL = '/api/v1';

interface ApiOptions extends Omit<RequestInit, 'body'> {
  body?: unknown;
  params?: Record<string, string | undefined>;
}

export class ApiError extends Error {
  status: number;

  constructor(status: number, message: string) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
  }
}

/**
 * Sanitize error messages extracted from API responses before surfacing to users.
 * Strips HTML tags, removes stack trace lines, collapses whitespace, and truncates.
 */
export function sanitizeErrorMessage(raw: string): string {
  // Strip HTML tags
  const noHtml = raw.replace(/<[^>]*>/g, '');
  // Remove lines that look like stack traces
  const lines = noHtml.split('\n');
  const noStack = lines
    .filter(line => !/^\s*at\s/i.test(line) && !/^\s*File\s/i.test(line))
    .join(' ')
    .trim();
  // Collapse excessive whitespace
  const collapsed = noStack.replace(/\s+/g, ' ');
  // Truncate to 200 characters
  return collapsed.length > 200 ? collapsed.slice(0, 200) + '…' : collapsed;
}

async function request<T>(path: string, options: ApiOptions = {}): Promise<T> {
  const { body, params, headers: customHeaders, ...rest } = options;

  const url = new URL(`${BASE_URL}${path}`, window.location.origin);

  if (params) {
    for (const [key, value] of Object.entries(params)) {
      if (value !== undefined) {
        url.searchParams.set(key, value);
      }
    }
  }

  const headers = new Headers(customHeaders);
  if (body !== undefined && !(body instanceof FormData)) {
    headers.set('Content-Type', 'application/json');
  }

  const response = await fetch(url.toString(), {
    ...rest,
    headers,
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });

  if (!response.ok) {
    const rawMessage = await response.text().catch(() => 'Request failed');
    const message = sanitizeErrorMessage(rawMessage);
    // Dispatch error event for toast system to pick up
    window.dispatchEvent(new CustomEvent('api:error', {
      detail: { message, status: response.status },
    }));
    throw new ApiError(response.status, message);
  }

  // 204 No Content — no body to parse (common for DELETE responses)
  if (response.status === 204) {
    return undefined as unknown as T;
  }

  return response.json() as Promise<T>;
}

export const api = {
  get: <T>(path: string, params?: Record<string, string | undefined>) =>
    request<T>(path, { method: 'GET', params }),

  post: <T>(path: string, body?: unknown) =>
    request<T>(path, { method: 'POST', body }),

  put: <T>(path: string, body?: unknown) =>
    request<T>(path, { method: 'PUT', body }),

  patch: <T>(path: string, body?: unknown) =>
    request<T>(path, { method: 'PATCH', body }),

  delete: <T>(path: string) => request<T>(path, { method: 'DELETE' }),
};
