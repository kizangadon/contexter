# Wave 1 Task Brief — Dependency + Simple Data Display Components

## Project Context
Contexter is a React SPA for managing AI agent memory, sessions, agents, skills, and analytics. This is Phase 4, building the UI component library.

**Tech Stack:** React 19, TypeScript 6 (strict), Vite 8, Tailwind CSS v4, Vitest + Testing Library, react-router v7

**Design Tokens:** V2-DEEP dark-only palette defined in `src/styles/tokens.css` as CSS custom properties. Use token variable names (e.g., `var(--color-accent)` not hardcoded colors). The accent color is `#7C5CFC`.

**Working Directory:** `/home/don/Code/contexter/contexter-web/`

**Existing Components:** `src/components/ui/Button.tsx` — use as reference for style patterns (Tailwind classes referencing design tokens, TypeScript interfaces, etc.)

## Your Task

Build 6 components **TDD-first**: write test file first, run `vitest run` (expect RED), write implementation, run `vitest run` (expect GREEN), then move to next component.

### Order to Build

1. **LoadingSkeleton** (`src/components/ui/LoadingSkeleton.tsx`)
   - Props: `variant` — `'card' | 'table-row' | 'text'`
   - Renders a placeholder pulse-animation div
   - `card`: rectangular block (h-32 rounded-lg)
   - `table-row`: horizontal bar with multiple cells
   - `text`: single line bar
   - All with `animate-pulse bg-bg-tertiary rounded` styling
   - TDD tests: renders card variant, renders table-row variant, renders text variant

2. **EmptyState** (`src/components/ui/EmptyState.tsx`)
   - Props: `icon` (LucideIcon component), `title` (string), `message` (string), `action` (optional ReactNode)
   - Renders a centered flex column with the icon (large, `--color-text-tertiary`), title (medium, `--color-text-primary`), message (small, `--color-text-secondary`), and action slot
   - TDD tests: renders icon, title, and message; renders action when provided

3. **Input** (`src/components/ui/Input.tsx`)
   - Props: extends `InputHTMLAttributes<HTMLInputElement>` with optional `label`, `error` (string)
   - Styled input with V2-DEEP tokens: bg `--color-bg-secondary`, border `--color-border`, focus border `--color-accent`, text `--color-text-primary`
   - When `error` is set, border turns `--color-error` and error text shown below
   - Optional label above the input
   - TDD tests: renders input, renders label, renders error message, shows error styling

4. **Tag** (`src/components/ui/Tag.tsx`)
   - Props: `label` (string), `color` (optional string — one of the V2-DEEP status colors: `success`, `warning`, `error`, `info`, `pending`, `offline`), `onRemove` (optional callback)
   - Small rounded badge with optional colored background
   - Max-width with `...` truncation at 50 chars via CSS (overflow-hidden text-ellipsis whitespace-nowrap max-w-[50ch])
   - When `onRemove` is provided, show X icon button on the right (use the `X` icon from lucide-react, small)
   - Default background: `bg-bg-tertiary` with `text-text-secondary`
   - Color backgrounds map to: `success` → bg-success/20 + text-success, `warning` → bg-warning/20 + text-warning, `error` → bg-error/20 + text-error, `info` → bg-info/20 + text-info, `pending` → bg-pending/20 + text-pending, `offline` → bg-offline/20 + text-offline
   - TDD tests: renders label, renders with color variant, shows remove button when onRemove provided, clicking remove fires callback, truncates text > 50 chars

5. **TimeframeFilter** (`src/components/ui/TimeframeFilter.tsx`)
   - Props: `value` (string), `onChange` (callback: (value: string) => void)
   - Dropdown/select with preset options: "Last 7 days" (`7d`), "Last 30 days" (`30d`), "Last 90 days" (`90d`), "All time" (`all`)
   - Optional "Custom" option (`custom`) → shows date-range picker (two date inputs when value is 'custom')
   - Styled with bg `--color-bg-secondary`, border `--color-border`, text `--color-text-primary`
   - TDD tests: renders with default value, selecting preset calls onChange, custom option shows date inputs

6. **EntityLink** (`src/components/ui/EntityLink.tsx`)
   - Props: `to` (string — route path), `children` (ReactNode), `type` (optional — `'session' | 'memory' | 'agent' | 'skill'`)
   - Renders as a `<Link>` from `react-router` (v7, import from `"react-router"`)
   - Color: `#7C5CFC` (via `text-accent`)
   - Hover: underline (via `hover:underline`)
   - Optional `type` prepends a small colored dot indicator
     - session → purple (bg-accent)
     - memory → green (bg-success)
     - agent → blue (bg-info)
     - skill → amber (bg-warning)
   - TDD tests: renders link with correct href, renders children, renders type indicator dot, hover shows underline

### Key Patterns (from existing Button.tsx)

- Import types from React (not separate import from Renderable etc.)
- Use `className` prop with default `''`
- Apply Tailwind classes referencing CSS custom properties via Tailwind's token names (`bg-surface`, `text-text-primary`, `border-border`, etc.)
- Export as named function `export function ComponentName(...)`

### Verification

After ALL 6 components are built with passing tests:
```bash
npx vitest run --reporter=verbose
```
Should show ALL tests passing (the new ones + existing Button tests).

### Skills to Load

Load each skill using the `skill` tool:

- `react-expert`
- `typescript-pro`
- `tailwind-css-patterns`
- `tdd`
- `javascript-testing-patterns`
- `incremental-implementation`
- `clean-code`
- `verification-before-completion`

### Handoff Report

Create a report file at `/home/don/Code/contexter/docs/tasks/wave1/handoff-report.md` containing:
1. Files created per component
2. TDD evidence for each (command output showing RED then GREEN)
3. Full `vitest run --reporter=verbose` output
4. Issues encountered
