# SPEC: Add backward-compatible token aliases

## Problem
The new `tokens.css` removed old token names (`--color-border`, `--color-surface`, `--color-success`, etc.) that are still referenced by 30+ `var()` calls across 8 page components. The Badge component uses Tailwind classes `bg-success`, `text-success`, etc. which Tailwind no longer generates (tokens are now `--color-status-success`).

## Fix
Add backward-compatible aliases in the `:root` block of `tokens.css` mapping each old name to its new equivalent. Also update the `:root` `--color-accent` alias (currently `--accent-muted` uses `#7C5CFC20` hex-alpha but some callers may reference `--color-accent` directly).

## Aliases to add
```css
:root {
  --color-border: var(--color-border-default);
  --color-surface: var(--color-surface-card);
  --color-success: var(--color-status-success);
  --color-error: var(--color-status-error);
  --color-warning: var(--color-status-warning);
  --color-info: var(--color-status-info);
  --color-pending: var(--color-status-pending);
  --color-offline: var(--color-status-offline);
  --color-bg-primary: var(--color-bg-base);
  --color-bg-secondary: var(--color-bg-elevated);
  --color-bg-tertiary: var(--color-bg-hover);
}
```

## Verification
```bash
cd /home/don/Code/contexter/contexter-web && npm run build 2>&1 | tail -5
```
Build must succeed with zero errors.
