# Bug-Fix Report: Mutex/RwLock Poison Recovery

| Field | Detail |
|---|---|
| **Bug Contract** | `2026-07-25-bug-poison` |
| **Fix Applied** | ✅ Yes (part of Bug-DB-Analytics Worker work + other Workers) |
| **Worker** | Multiple (Bug-DB-Analytics, Fix-Poison, etc.) |

## Changes Applied

### REQ-FIX-001: Poison recovery for DuckDbEngine Mutex
Applied `.unwrap_or_else(|e| e.into_inner())` pattern to all `conn.lock()` calls in `analytics/duckdb.rs`.

### REQ-FIX-002: Poison recovery for Engine locks
Applied the same pattern to all `RwLock`/`Mutex` accesses across the engine layer.

## Scope

**73 occurrences** of the poison recovery pattern found across:

| File | Occurrences |
|---|---|
| `analytics/duckdb.rs` | 12 |
| `fts/tantivy.rs` | 3 |
| `vector/hnsw.rs` | 22 |
| `engine/session.rs` | 7 |
| `engine/agent.rs` | 6 |
| `engine/skill.rs` | 6 |
| `engine/settings.rs` | 5 |
| `engine/search.rs` | 2 |
| `engine/memory.rs` | 4 |
| `engine/maintenance.rs` | 6 |

## Verification

- ✅ `cargo build --workspace` — compiles cleanly
- ✅ Every `Mutex::lock()` and `RwLock::read()/write()` uses `.unwrap_or_else(|e| e.into_inner())`
