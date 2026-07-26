import { describe, expect, it } from 'vitest';
import { sanitizeErrorMessage } from './client';

describe('sanitizeErrorMessage', () => {
  it('passes through plain text messages unchanged', () => {
    expect(sanitizeErrorMessage('Not found')).toBe('Not found');
  });

  it('strips HTML tags from messages', () => {
    // Tags are removed; adjacent tags without whitespace between them
    // may produce concatenated text. This is expected — the sanitizer
    // does not inject spaces around stripped tags.
    const html = '<h1>Error</h1><p>Something went <strong>wrong</strong></p>';
    expect(sanitizeErrorMessage(html)).toBe('ErrorSomething went wrong');
  });

  it('removes stack trace lines', () => {
    const msg = [
      'Internal server error',
      '  at handleRequest (/app/server.ts:42:10)',
      '    at dispatch (/app/router.ts:105:14)',
      'Cause: database timeout',
    ].join('\n');
    expect(sanitizeErrorMessage(msg)).toBe('Internal server error Cause: database timeout');
  });

  it('removes File-prefixed trace lines', () => {
    const msg = [
      'Something broke',
      '  File "/app/core.py", line 50, in process',
      '  File "/app/core.py", line 30, in _run',
    ].join('\n');
    expect(sanitizeErrorMessage(msg)).toBe('Something broke');
  });

  it('collapses excessive whitespace', () => {
    const msg = 'Error:    too    many    spaces';
    expect(sanitizeErrorMessage(msg)).toBe('Error: too many spaces');
  });

  it('truncates messages longer than 200 characters', () => {
    const longMsg = 'x'.repeat(250);
    const result = sanitizeErrorMessage(longMsg);
    expect(result.length).toBe(201); // 200 chars + '…'
    expect(result.endsWith('…')).toBe(true);
    expect(result.slice(0, 200)).toBe('x'.repeat(200));
  });

  it('does not truncate messages at 200 characters', () => {
    const msg = 'a'.repeat(200);
    expect(sanitizeErrorMessage(msg)).toBe(msg);
  });

  it('handles empty messages', () => {
    expect(sanitizeErrorMessage('')).toBe('');
  });

  it('handles messages with only HTML tags', () => {
    expect(sanitizeErrorMessage('<html><body></body></html>')).toBe('');
  });

  it('handles messages with only stack traces', () => {
    const traces = '  at main (app.ts:1)\n  at run (bootstrap.ts:10)';
    expect(sanitizeErrorMessage(traces)).toBe('');
  });

  it('collapses newlines within non-stack content', () => {
    const msg = 'Line one\nLine two\n\nLine three';
    const result = sanitizeErrorMessage(msg);
    // Newlines between non-stack lines become spaces, then collapsed
    expect(result).toBe('Line one Line two Line three');
  });
});
