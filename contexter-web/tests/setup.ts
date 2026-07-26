import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/react';
import { afterAll, afterEach, beforeAll } from 'vitest';
import { server } from './mocks/server';

// Start MSW server before all tests
beforeAll(() => server.listen({ onUnhandledRequest: 'warn' }));

// Reset handlers after each test (clean up request handlers)
afterEach(() => {
  cleanup();
  server.resetHandlers();
  // Remove portal content rendered to document.body via createPortal
  document.body.innerHTML = '';
});

// Close server after all tests
afterAll(() => server.close());
