---
phase: 1
slug: core-object-system
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-17
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` with `cargo test` |
| **Config file** | none — standard Cargo test runner |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 1 | OBJ-01 | unit | `cargo test object_pool` | ❌ W0 | ⬜ pending |
| 01-01-02 | 01 | 1 | OBJ-02 | unit | `cargo test object_pool::tests::lifecycle` | ❌ W0 | ⬜ pending |
| 01-02-01 | 02 | 1 | OBJ-03 | unit | `cargo test cell_grid` | ❌ W0 | ⬜ pending |
| 01-02-02 | 02 | 1 | OBJ-04 | unit | `cargo test cell_grid::tests::linkage` | ❌ W0 | ⬜ pending |
| 01-03-01 | 03 | 2 | OBJ-05 | integration | `cargo test` | ✅ (260 existing) | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/engine/pool.rs` — ObjectPool struct with allocation tests
- [ ] `src/engine/pool.rs::tests` — pool lifecycle, free list, two-tier allocation tests
- [ ] `src/engine/cell_grid.rs::tests` — cell linkage insert/remove/move tests

*Existing 260 tests cover regression for OBJ-05 (UnitCoordinator migration).*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Allocation order matches original binary | OBJ-02 | Requires Frida trace comparison | Run allocation sequence, compare order with Frida trace fixture |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
