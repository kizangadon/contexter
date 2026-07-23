# Contexter — Specification Hub

**Date:** 2026-07-23
**Status:** Draft
**Repository:** [contexter](../)

---

## 1. Purpose

This document is the **single entry point** for all Contexter specifications. It indexes every design and architecture document in the project, defines how they relate to each other, and provides the dependency order for implementation.

---

## 2. Spec Tree

```
contexter-specification-hub.md          ← YOU ARE HERE — index, navigation, dependencies
│
├── contexter-system-architecture.md    ← Tech architecture: Rust core, Python layer,
│                                          React UI, data model, storage, APIs,
│                                          WAL, vector index, versioning, analytics
│
└── contexter-ui-design.md             ← UI design entry point (parent of layout-
                                          level sub-specs below)
    │
    ├── 2026-07-23-phase-2-core-ui.md             ← 10 pages: Dashboard, Sessions,
    │                                                  Memories, Agents, Skills,
    │                                                  Efficiency Mapper
    │
    ├── 2026-07-23-efficiency-mapper-details.md   ← 6 detail pages: Memory Usage,
    │                                                  Session Activity, Agent
    │                                                  Performance, etc.
    │
    ├── 2026-07-23-system-analytics.md            ← 7 pages: Overview, Health,
    │                                                  Performance, Resources, Costs,
    │                                                  Model Detail, Services
    │
    ├── 2026-07-23-settings-configuration.md      ← 8 sections: General, Storage,
    │                                                  MCP, LLM, Notifications,
    │                                                  Agents & Skills, Analytics,
    │                                                  Data Management
    │
    └── 2026-07-23-standalone-features.md         ← 9 pages: Notifications, Feedback,
                                                       Onboarding, API Playground,
                                                       Search, Export, Correlation,
                                                       Audit
```

---

## 3. Document Map

| Document | Covers | Implementation Dependency |
|---|---|---|
| `2026-07-23-contexter-system-architecture.md` | Rust core, Python layer, React UI structure, data model, storage engine, WAL, vector index, PyO3 bridge, API surface, MCP server, config, edge cases | **1st** — must be read before any implementation |
| `2026-07-23-contexter-ui-design.md` | Global navigation, shared component library, page hierarchy, UI patterns | **2nd** — references shared components that need to be built first |
| `2026-07-23-phase-2-core-ui.md` | Dashboard, Memory Explorer, Memory Detail, Session Manager, Session Detail, Agent Registry, Agent Detail, Skill Registry, Skill Detail, Efficiency Mapper | **3rd** — depends on AppShell + shared components |
| `2026-07-23-efficiency-mapper-details.md` | Memory Usage, Session Activity, Agent Performance, Skill Effectiveness, Token Usage, Correlation Matrix | **4th** — drill-down pages from Efficiency Mapper |
| `2026-07-23-system-analytics.md` | Analytics Overview, System Health, Performance Trends, Resource Usage, Cost & Token Analytics, Model Detail, Service Status | **5th** — depends on shared chart components |
| `2026-07-23-settings-configuration.md` | 8 settings sections (General, Storage, MCP Server, LLM Providers, Notifications, Agents & Skills, Analytics, Data Management) | **6th** — independent feature blocks |
| `2026-07-23-standalone-features.md` | Notification Center, Feedback & Bug Reporter, Onboarding, API Playground, Global Search, Data Export & Reports, Cross-Session Correlation, Versioning & Audit Trail | **7th** — lowest priority, independent of each other |

---

## 4. How to Read This Spec Tree

1. **Start with the architecture doc** — understand the system's technical foundations before touching any UI.
2. **Read the UI design doc** — understand the navigation structure and shared component library.
3. **Pick a sub-spec** — each is self-contained and can be implemented independently once shared components exist.
4. **Refer to mockups** — every page table in each sub-spec references an HTML mockup file at `.superpowers/brainstorm/1389270-1784815136/content/` for the approved visual design.
5. **Refer to the design system** — all color, typography, and spacing tokens are in `docs/design/V2-DEEP-design-system.md`.

---

## 5. Related Documents

| Document | Location |
|---|---|
| Design System Tokens | `docs/design/V2-DEEP-design-system.md` |
| UI Mockup Files | `.superpowers/brainstorm/1389270-1784815136/content/` |
| Repository Root | `../` |

---

## 6. Status Legend

| Status | Meaning |
|---|---|
| **Draft** | Written, not yet reviewed by user |
| **Reviewed** | User has reviewed, may have pending changes |
| **Approved** | User approved. Ready for implementation planning. |
| **In Progress** | Implementation underway |
| **Complete** | Implementation verified and shipped |
