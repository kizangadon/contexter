# Contexter — V2 Deep Design System

> **Status:** `APPROVED` · **Version:** `v1.0`
> **Inspired by:** Stripe Dashboard · Warm Dark + Purple Accent

---

## Table of Contents

- [Philosophy](#philosophy)
- [Color Palette](#color-palette)
- [Typography](#typography)
- [Spacing & Sizing](#spacing--sizing)
- [Border Radius & Shadows](#border-radius--shadows)
- [Cards & Surfaces](#cards--surfaces)
- [Data Visualization](#data-visualization)
- [Status & Semantic Colors](#status--semantic-colors)
- [Dark Mode (Only Mode)](#dark-mode-only-mode)
- [Motion & Animation](#motion--animation)
- [Iconography](#iconography)
- [Component Primitives](#component-primitives)

---

## Philosophy

V2 Deep is a **Stripe-inspired** dark design system built for data-dense operational dashboards. It prioritizes clarity, hierarchy, and a premium feel — not visual decoration.

**Core principles:**
- **Warm, not cold.** The dark base (#181716) has warmth. Pure black is never used.
- **Purple as identity, not decoration.** #7C5CFC is used intentionally — for interactive elements, active states, and data highlights.
- **Information, not chrome.** Cards are separated by background tint, not borders. Chrome (lines, dividers) is minimized.
- **Generous breathing room.** Spacing is deliberate and consistent. Content is never cramped.
- **Refined, not flashy.** Subtle shadows, smooth curves (8px), muted secondary text. The design gets out of the way.

---

## Color Palette

### Base Colors

| Token | Hex | Usage |
|-------|-----|-------|
| `--bg-base` | `#181716` | Page background, outermost canvas |
| `--bg-elevated` | `#1F1E1D` | Card / surface background |
| `--bg-hover` | `#252423` | Card hover, dropdown hover |
| `--bg-active` | `#2A2928` | Active/selected state background |
| `--bg-inset` | `#131211` | Input fields, code blocks, inset areas |

### Accent / Brand

| Token | Hex | Usage |
|-------|-----|-------|
| `--accent` | `#7C5CFC` | Primary buttons, active links, focus rings |
| `--accent-hover` | `#6A4DE0` | Button hover |
| `--accent-muted` | `#7C5CFC20` | Subtle accent background (selected rows, etc.) |
| `--accent-glow` | `#7C5CFC40` | Glow effects, focus rings |

### Text

| Token | Hex | Usage |
|-------|-----|-------|
| `--text-primary` | `#F2F0EE` | Primary body text, headings |
| `--text-secondary` | `#A09E9B` | Secondary text, metadata, table body |
| `--text-tertiary` | `#6F6D6B` | Placeholder text, disabled labels |
| `--text-inverse` | `#181716` | Text on accent-colored backgrounds |
| `--text-link` | `#7C5CFC` | Hyperlinks |
| `--text-link-hover` | `#9B82FF` | Hyperlink hover |

### Borders & Lines

| Token | Hex | Usage |
|-------|-----|-------|
| `--border-subtle` | `#2A2928` | Subtle dividers, table row separators |
| `--border-default` | `#343231` | Input borders, card borders (when needed) |
| `--border-accent` | `#7C5CFC` | Focus rings, active input borders |

### Surface Gradients

| Token | Value | Usage |
|-------|-------|-------|
| `--gradient-card` | `linear-gradient(135deg, #1F1E1D 0%, #1D1C1B 100%)` | Optional subtle card gradient |
| `--gradient-accent` | `linear-gradient(135deg, #7C5CFC 0%, #6344E0 100%)` | Primary button gradient |
| `--gradient-accent-glow` | `radial-gradient(ellipse at center, #7C5CFC20 0%, transparent 70%)` | Background glow for accent areas |

---

## Typography

### Font Family

| Token | Value | Usage |
|-------|-------|-------|
| `--font-sans` | `'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif` | UI text |
| `--font-mono` | `'JetBrains Mono', 'SF Mono', 'Fira Code', monospace` | Code, metrics, data values |
| `--font-display` | `'Inter', sans-serif` | Headings (same as sans, weight differentiates) |

### Type Scale

| Token | Size | Weight | Line Height | Usage |
|-------|------|--------|-------------|-------|
| `--text-xs` | 11px | 500 | 16px | Tiny labels, badge text |
| `--text-sm` | 12px | 500 | 18px | Table cells, metadata, secondary text |
| `--text-base` | 14px | 450 | 22px | Body text, most UI text |
| `--text-lg` | 16px | 500 | 24px | Card titles, subsection headings |
| `--text-xl` | 20px | 600 | 28px | Section headings, dialog titles |
| `--text-2xl` | 24px | 600 | 32px | Page titles |
| `--text-3xl` | 32px | 650 | 40px | Dashboard hero metrics |

### Font Features

```css
font-feature-settings: 'cv02', 'cv03', 'cv04', 'cv11';  /* Inter stylistic alternates */
/* Tabular numbers enabled for data displays */
font-variant-numeric: tabular-nums;
```

---

## Spacing & Sizing

### Space Scale

Based on multiples of 4px, with generous rhythm for data density.

| Token | Value | Usage |
|-------|-------|-------|
| `--space-1` | 4px | Tight gaps, icon margins |
| `--space-2` | 8px | Card padding (compact), avatar gaps |
| `--space-3` | 12px | Button padding, input padding |
| `--space-4` | 16px | Card padding (default), section spacing |
| `--space-5` | 20px | Between card groups |
| `--space-6` | 24px | Major section gaps |
| `--space-8` | 32px | Page section separation |
| `--space-10` | 40px | Page padding |
| `--space-12` | 48px | Large page breaks |
| `--space-16` | 64px | Full section separation |

### Layout

| Token | Value | Usage |
|-------|-------|-------|
| `--max-content-width` | 1440px | Maximum page width |
| `--sidebar-width` | 240px | Navigation sidebar width |
| `--sidebar-collapsed` | 60px | Collapsed sidebar |
| `--topbar-height` | 56px | Top navigation bar height |

---

## Border Radius & Shadows

### Border Radius

| Token | Value | Usage |
|-------|-------|-------|
| `--radius-sm` | 4px | Inputs, buttons, small components |
| `--radius-md` | 8px | Cards, dialogs, dropdowns |
| `--radius-lg` | 12px | Modals, large surfaces |
| `--radius-xl` | 16px | Page-level containers |
| `--radius-full` | 9999px | Pills, badges, avatars |

### Shadows

| Token | Value | Usage |
|-------|-------|-------|
| `--shadow-sm` | `0 1px 2px rgba(0,0,0,0.3)` | Subtle card elevation |
| `--shadow-md` | `0 4px 12px rgba(0,0,0,0.4)` | Elevated cards, dropdowns |
| `--shadow-lg` | `0 8px 30px rgba(0,0,0,0.5)` | Modals, dialogs |
| `--shadow-accent` | `0 0 20px #7C5CFC30` | Accent glow (focus, active states) |

All shadows on dark surfaces use black-based shadows, not colored.

---

## Cards & Surfaces

Cards are the primary content container. They use **background-only separation** — no borders.

| Token | Value | Usage |
|-------|-------|-------|
| `--surface-card` | `#1F1E1D` | Default card surface |
| `--surface-card-alt` | `#222120` | Alternative card (for grid differentiation) |
| `--surface-card-hover` | `#252423` | Card hover state |
| `--surface-card-accent` | `#1F1D24` | Card with accent tint (for featured data) |

**Card anatomy:**
```
┌─────────────────────────────────┐
│  #1F1E1D surface                │  ← rounded 8px
│                                 │
│  Header text  (--text-lg)       │
│  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─  │  ← --border-subtle divider (optional)
│  Content area                   │
│  padding: --space-4 (16px)      │
│                                 │
└─────────────────────────────────┘
```

---

## Data Visualization

### Chart Colors

For Recharts / chart components, extending the accent palette:

| Token | Hex | Usage |
|-------|-----|-------|
| `--chart-1` | `#7C5CFC` | Primary data series (accent) |
| `--chart-2` | `#4FC3F7` | Secondary series |
| `--chart-3` | `#66BB6A` | Positive trends, success |
| `--chart-4` | `#FFA726` | Warning, attention |
| `--chart-5` | `#EF5350` | Negative trends, errors |
| `--chart-6` | `#AB47BC` | Additional series |
| `--chart-7` | `#26C6DA` | Additional series |
| `--chart-8` | `#8D6E63` | Baseline, muted series |

### Chart Grid & Axes

| Token | Value | Usage |
|-------|-------|-------|
| `--chart-grid` | `#2A2928` | Grid lines |
| `--chart-axis` | `#6F6D6B` | Axis labels |
| `--chart-zero` | `#343231` | Zero line |

---

## Status & Semantic Colors

| Token | Hex | Usage |
|-------|-----|-------|
| `--status-success` | `#4CAF50` | Success, healthy, up |
| `--status-warning` | `#FF9800` | Warning, degraded, attention |
| `--status-error` | `#F44336` | Error, critical, down |
| `--status-info` | `#42A5F5` | Info, neutral notification |
| `--status-pending` | `#FFA726` | Pending, in-progress |
| `--status-offline` | `#6F6D6B` | Inactive, offline |

**Semantic backgrounds** (for status badges, pills):

| Token | Value | Usage |
|-------|-------|-------|
| `--bg-status-success` | `#4CAF5020` | Success badge bg |
| `--bg-status-warning` | `#FF980020` | Warning badge bg |
| `--bg-status-error` | `#F4433620` | Error badge bg |
| `--bg-status-info` | `#42A5F520` | Info badge bg |

---

## Dark Mode (Only Mode)

Contexter is **dark-first**. There is no light mode in v1. The entire design system is built around a single dark theme.

If light mode is added in the future, the tokens above would be inverted:
- Light backgrounds become white / near-white
- Text colors invert (dark text on light bg)
- Shadows shift to lighter, more transparent
- Accent remains #7C5CFC

---

## Motion & Animation

| Token | Value | Usage |
|-------|-------|-------|
| `--ease-out` | `cubic-bezier(0.16, 1, 0.3, 1)` | UI transitions (cards, modals) |
| `--ease-in-out` | `cubic-bezier(0.65, 0, 0.35, 1)` | Page transitions, tabs |
| `--duration-fast` | 150ms | Micro-interactions (hover, focus) |
| `--duration-normal` | 250ms | Standard transitions |
| `--duration-slow` | 400ms | Page transitions, modals |

---

## Iconography

- **Library:** Lucide icons (consistent weight, open source)
- **Style:** Outlined, 1.5px stroke, 16px default size
- **Color:** Inherits from text color by default; can use `--accent` for active states

---

## Component Primitives

### Button

| Variant | Background | Text | Hover | Active |
|---------|-----------|------|-------|--------|
| Primary | `--gradient-accent` | `#FFFFFF` | `--accent-hover` | Slightly darker |
| Secondary | `transparent` | `--text-primary` | `--bg-hover` | `--bg-active` |
| Ghost | `transparent` | `--text-secondary` | `--bg-hover` | `--bg-active` |
| Danger | `--status-error` | `#FFFFFF` | Darker red | Darkest red |

**Sizing:**
- Default: 32px height, `--space-3` horizontal padding, `--text-base`
- Small: 24px height, `--space-2` horizontal padding, `--text-sm`
- Large: 40px height, `--space-4` horizontal padding, `--text-lg`

### Badge / Pill

- Background: semantic status bg (`--bg-status-*`)
- Text: semantic status color (`--status-*`)
- Radius: `--radius-full`
- Padding: 2px 8px
- Font: `--text-xs`, uppercase

### Input

- Background: `--bg-inset` (#131211)
- Border: `--border-default` (#343231)
- Focus border: `--border-accent` (#7C5CFC)
- Text: `--text-primary`
- Placeholder: `--text-tertiary`
- Radius: `--radius-sm` (4px)
- Height: 32px default

### Table

- Header bg: `--bg-base` (#181716)
- Header text: `--text-secondary` (12px, uppercase tracking)
- Row bg: `--surface-card` (#1F1E1D)
- Row hover: `--bg-hover` (#252423)
- Row selected: `--accent-muted` (#7C5CFC20)
- Border: `--border-subtle` (#2A2928) — horizontal only, no vertical lines
- Cell padding: 12px 16px

### Navigation Sidebar

- Background: same as `--bg-base` (#181716)
- Width: 240px (expanded), 60px (collapsed)
- Item padding: 8px 16px
- Active item: `--accent-muted` bg + `--accent` left border (2px)
- Hover item: `--bg-hover`
- Icon size: 18px
- Section labels: `--text-xs`, uppercase, `--text-tertiary`

### Dialog / Modal

- Overlay: `rgba(0, 0, 0, 0.6)`
- Surface: `--bg-elevated` (#1F1E1D)
- Radius: `--radius-lg` (12px)
- Shadow: `--shadow-lg`
- Title: `--text-xl`
- Close button: ghost variant

---

## File Structure (for implementation)

```css
/* contexter-ui/src/styles/tokens.css */
:root {
  /* Base */
  --bg-base: #181716;
  --bg-elevated: #1F1E1D;
  /* ... all tokens above ... */
}
```

All components reference these CSS custom properties. No hardcoded color values outside the token definition.

---

*Design system approved for Contexter v1.0 · July 2026*
