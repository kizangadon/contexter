# Performance Review Report

# Contexter Phase 1 — Workspace Restructure

> Structural audit of the contexter-core workspace restructure: 9,175 lines of Rust organized into a `[workspace]` with `contexter-core` as the sole member. No logic changes — pure reorganisation of module boundaries and crate topology.

**Verdict:** CONDITIONAL PASS (class: amber)

2026-07-24 · 6 benchmarks · **Performance Benchmarker**

---

## 01 · Performance Overview

| Metric | Value |
|---|---|
| Source lines (Rust) | 9,175 across 42 source files |
| Largest file | `storage/rocksdb.rs` — 2,011 lines |
| Modules | 15 public, 4 feature-gated, 6 Phase-2 stubs |
| Workspace members | 1 (`contexter-core`) |
| Dependency count | 11 direct + 3 optional |
| Crate types | `lib` + `cdylib` + `bin` |
| Orphaned lockfiles | **1** (`contexter-core/Cargo.lock`) — stale, untracked |
| Glob re-export chains | **2** (`pub use models::*` in lib.rs → `pub use agent::*` etc. in mod.rs) |
| Inline test lines | ~1,300 in `engine/mod.rs` (86% of that file) |

> **Analysis Scope**
> This review focuses on build-performance implications of the workspace restructure: module boundaries, re-export cascades, recompilation isolation, binary size impact, and test-performance characteristics. No runtime benchmarks apply since this is a purely structural change with zero logic diffs. The SPEC/ACCEPTANCE/EDGE_CASES documentation is the reference contract; this report verifies the restructure does not introduce performance regressions.

---

## 02 · Benchmark Results

### Benchmark 1: Workspace Topology — Build Isolation

| Property | Finding |
|---|---|
| Members | `["contexter-core"]` — single-member workspace |
| Resolver | `resolver = "2"` — correct default for edition 2021 |
| Redundant recompilation | **None.** Single member means no cross-crate recompilation boundaries today. |
| Future expansion risk | Once Phase 2 adds members, the `pub use models::*` glob chain will cause cascading rebuilds across all downstream workspace members when any model file changes. |

**Impact:** ✅ LOW — no immediate concern. The workspace preamble (3 lines) is minimal overhead.

### Benchmark 2: Module Boundary Analysis — Dependency Graph

```
lib.rs
 ├── cache           → types, error
 ├── cli             → engine, error, models (via `use crate::*`)
 ├── compression     → stdlib only (self-contained)
 ├── engine          → cache, error, storage, models
 ├── error           → stdlib only (self-contained)
 ├── models          → stdlib only (self-contained)
 ├── storage         → error, models
 ├── bridge [pyo3]   → engine, error, models
 ├── crdt            → stdlib only (stub)
 ├── telemetry       → stdlib only (stub)
 ├── util            → stdlib only (stub)
 ├── versioning      → stdlib only (stub)
 ├── wal             → stdlib only (stub)
 ├── analytics [stub]
 ├── fts [stub]
 └── vector [stub]
```

**No circular dependencies detected.** The dependency graph is a directed acyclic graph (DAG) — good for incremental compilation. The heaviest fan-out nodes are `models` (imported by 4 modules) and `error` (imported by 3 modules).

**Impact:** ✅ NONE — clean architecture.

### Benchmark 3: Re-export Cascade Analysis

The two-level glob chain in `lib.rs`:

```rust
// lib.rs line 49
pub use models::*;   // ← glob re-exports EVERYTHING from models

// models/mod.rs lines 19-28
pub use agent::*;    // ← inner glob re-exports
pub use audit::*;
pub use memory::*;
// ... 8 more globs
```

**Why this matters:** Every symbol from every `models/` submodule becomes part of the crate's public API. The Rust compiler cannot determine which symbols are actually used by downstream consumers, so **any change to any model file** invalidates the entire crate's compilation unit. With a single-member workspace today this has no effect, but when Phase 2 introduces downstream crates, these crates will fully recompile on every model change — even for fields they never touch.

| Chained file | Symbols exported |
|---|---|
| `models/agent.rs` (138 lines) | `Agent`, `AgentFilter`, `AgentPatch`, `AgentStatus`, `NewAgent` |
| `models/memory.rs` (173 lines) | `Memory`, `MemoryFilter`, `MemoryPatch`, `MemoryType`, `NewMemory`, `MemorySearchQuery` |
| `models/session.rs` (197 lines) | `Session`, `SessionFilter`, `SessionPatch`, `SessionStatus`, `NewSession` |
| `models/audit.rs` (104 lines) | `AuditEntry`, `AuditFilter`, `NewAuditEntry` |
| `models/skill.rs` (88 lines) | `Skill`, `SkillFilter`, `SkillPatch`, `NewSkill` |
| `models/settings.rs` (38 lines) | `SettingValue`, `NewSetting` |
| `models/correlation.rs` (25 lines) | `CorrelationId` |
| `models/feedback.rs` (23 lines) | `FeedbackScore` |
| `models/notification.rs` (24 lines) | `Notification` |
| `models/telemetry.rs` (25 lines) | `TelemetrySample` |

**Impact:** ⚠️ MEDIUM — zero cost today but a latent recompilation cascade when the workspace grows.

### Benchmark 4: Conditional Compilation Boundaries

| Feature gate | Module | Lines | Gated correctly? |
|---|---|---|---|
| `#[cfg(feature = "python")]` | `bridge.rs` | 963 | ✅ — behind `cfg(feature = "python")` |
| `#[cfg(feature = "python")]` | `pyo3` dependency | N/A | ✅ — optional dep |
| `#[cfg(not(target_os = "windows"))]` | `zstd`, `lz4` deps | N/A | ✅ — platform-gated |
| `compression` feature | `zstd + lz4` | N/A | ✅ — both optional |
| Phase-2 stubs `analytics`, `fts`, `vector` | 5-line stubs | 15 total | ✅ — always compiled but minimal |

Conditionally compiled code is properly isolated. `bridge.rs` (963 lines) only compiles when `--features python` is active, saving ~10% of crate compilation in the default configuration.

**Impact:** ✅ NONE — properly gated.

### Benchmark 5: Orphaned Lockfile — `contexter-core/Cargo.lock`

**Finding:** An untracked, stale `Cargo.lock` (34 KB, 1,379 lines) exists at `contexter-core/Cargo.lock`. Its content differs from the workspace root `Cargo.lock` (same deps, different lock ordering). This is a build artifact from an independent `cargo build` inside the member directory.

| Property | Workspace Root | Member (orphaned) |
|---|---|---|
| Path | `Cargo.lock` | `contexter-core/Cargo.lock` |
| Size | 36,294 bytes | 35,689 bytes |
| SHA256 | `246fd77...` | `13c5639...` |
| Git tracked | ✅ Yes | ❌ No (untracked) |
| Used during build | ✅ Yes | ❌ No |

While this orphaned file does **not** affect builds (Cargo always resolves dependencies from the workspace root), it can:
- Confuse developers running `cargo build` or `cargo check` inside `contexter-core/` (Cargo may warn about workspace root mismatch)
- Cause spurious CI diff noise if accidentally committed
- Suggests a stale build cache that wastes ~35 KB of disk

**Impact:** ⚠️ LOW — not a build blocker, but should be cleaned up.

### Benchmark 6: Test Performance — Inline vs Integration

| Test location | Lines | Type | Compile cost |
|---|---|---|---|
| `engine/mod.rs` (inline `#[cfg(test)]`) | ~1,300 | Unit | Compiled with library on `cargo test` |
| `tests/integration_test.rs` | 1,086 | Integration | Compiled separately; links against lib |
| `storage/mod.rs` (inline) | ~20 | Unit | Minimal |
| `models/` inline tests | 0 | — | None (no inline tests in models) |

The inline tests in `engine/mod.rs` represent **86% of that file's content**. Every `cargo test --lib` recompiles all 1,300 test lines alongside the ~200 implementation lines. This test-code-to-implementation-code ratio (6.5:1) is unusually high and adds measurable compile overhead during test cycles.

A refactor to move these tests to `tests/` or to a separate `engine/tests/` module would reduce `cargo check` iteration time for developers working on engine implementation.

**Impact:** ⚠️ LOW — compile-time overhead during `cargo test`, but runtime execution is fast (each test creates a RocksDB instance in a tempdir).

---

## 03 · Performance Bottlenecks

### [B-1] Orphaned `contexter-core/Cargo.lock` (LOW severity)

- **File:** `contexter-core/Cargo.lock` (untracked, 35 KB)
- **Detail:** A stale, second `Cargo.lock` lives inside the workspace member directory. It was generated when the crate was built as a standalone unit before the workspace restructure. It is untracked, differs from the root lockfile, and could cause confusion. While it doesn't affect workspace builds, it should be removed to avoid accidental commits or developer confusion.
- **Fix:** `rm contexter-core/Cargo.lock` and optionally add `contexter-core/Cargo.lock` to `.gitignore` as an insurance policy.

### [B-2] Glob re-export cascade (`pub use models::*`) (MEDIUM severity)

- **Files:** `lib.rs:49`, `models/mod.rs:19-28`
- **Detail:** The two-level glob re-export chain exposes every model type as part of the public API surface. While benign today (single workspace member), this creates a wide recompilation boundary: changing any field in any model file invalidates all downstream consumers of `contexter_core`. When Phase 2 adds multiple workspace members depending on `contexter-core`, a model change will cascade-rebuild every consuming crate.
- **Fix:** Replace `pub use models::*` with explicit re-exports for only the types actually consumed by external callers. The integration test file (`tests/integration_test.rs`) shows the current usage — 14 types: `AgentFilter`, `AgentStatus`, `AuditFilter`, `CacheConfig`, `Engine`, `MemoryFilter`, `MemoryPatch`, `MemorySearchQuery`, `MemoryType`, `NewAgent`, `NewAuditEntry`, `NewMemory`, `NewSession`, `NewSkill`, `Session`, `SessionFilter`, `SessionPatch`, `SessionStatus`, `SkillFilter`, `StorageConfig`. These can be explicitly listed.

### [B-3] Engine inline test density (LOW severity)

- **File:** `engine/mod.rs` — 1,300 test lines / 1,519 total
- **Detail:** 86% of the engine module is test code. Every `cargo test --lib` compiles this alongside production code. Moving the majority of these tests to `tests/engine.rs` (integration tests) would improve `cargo check` iteration speed for engine developers.
- **Fix:** Extract unit tests to a separate test file. The tests that use `setup()` already demonstrate they can be integration-style tests (they access Engine through its public API).

---

## 04 · Optimization Recommendations

> **High Impact**
> None — this is a structural restructure with zero logic changes. No runtime performance regression exists.

> **Medium Impact**
> 1. **Explicit re-exports over globs** — Replace `pub use models::*` in `lib.rs` and the individual `pub use agent::*` chains in `models/mod.rs` with explicit symbol exports. This narrows the recompilation boundary for future workspace members. Estimated effort: 15 minutes.
> 2. **Remove orphaned Cargo.lock** — Delete `contexter-core/Cargo.lock` and optionally add it to `.gitignore`. This prevents accidental commits and CI noise. Estimated effort: 1 minute.

> **Quick Wins**
> 1. **Add `Cargo.lock` to .gitignore at the member level** — Or more precisely, add `/Cargo.lock` to a `.gitignore` inside `contexter-core/` to prevent any future spurious lockfile generation inside member directories. Alternatively, rely on workspace-level discipline.
> 2. **Consider splitting engine tests** — When time permits, extract the 1,300 lines of inline `#[cfg(test)]` from `engine/mod.rs` into `tests/engine.rs` for faster `cargo check --lib` feedback. This is a developer-ergonomics improvement, not a production perf issue.

---

## Summary of Findings

| ID | Severity | Finding | Status |
|---|---|---|---|
| B-1 | LOW | Orphaned `contexter-core/Cargo.lock` (untracked, stale) | 🔴 OPEN |
| B-2 | MEDIUM | Glob re-export `pub use models::*` creates wide recompilation boundary for future workspace members | 🔴 OPEN |
| B-3 | LOW | 86% of engine/mod.rs is test code, inflating `cargo test --lib` compile time | 🔴 OPEN |

**Overall assessment:** This restructure introduces zero runtime performance regressions — the build output binary is logically identical to the unstructured equivalent. The workspace topology is clean (DAG module graph, proper feature gates). Two findings concern future-proofing (glob exports, orphaned lockfile) rather than current-state problems. One finding (test density) is developer-ergonomics.

**Verdict: CONDITIONAL PASS** — Address B-2 (glob re-exports) before the next workspace members are added. B-1 and B-3 are low-severity and can be deferred but should be documented.

---

_Generated by **Performance Benchmarker** · 2026-07-24 · Validation Contract: `contexter-phase1-restructure`_
