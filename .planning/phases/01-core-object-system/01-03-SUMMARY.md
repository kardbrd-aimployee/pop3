---
phase: 01-core-object-system
plan: 03
subsystem: units/coordinator
tags: [object-pool, migration, coordinator, cell-grid]
dependency_graph:
  requires: [01-01, 01-02]
  provides: [pool-backed-coordinator, units-accessor]
  affects: [app.rs, sprites/mod.rs]
tech_stack:
  added: []
  patterns: [compatibility-shim, accessor-method]
key_files:
  created: []
  modified:
    - src/engine/units/coordinator.rs
    - src/engine/objects/pool.rs
    - src/engine/objects/cell_grid.rs
    - src/render/app.rs
    - src/render/sprites/mod.rs
decisions:
  - Kept Vec<Unit> as compatibility shim rebuilt from pool, avoiding risky all-at-once tick() migration
  - Made units field private with pub fn units() accessor to enforce encapsulation
metrics:
  duration: ~3 min
  completed: 2026-03-18
---

# Phase 01 Plan 03: UnitCoordinator Migration Summary

UnitCoordinator owns ObjectPool + CellGrid; load_level allocates from pool; all 9+ external access sites use units() accessor instead of direct field access.

## What Was Done

### Task 1: Add pool and cell_grid to UnitCoordinator (c628c80)

- Added `pool: ObjectPool`, `cell_grid: CellGrid`, `person_handles: Vec<ObjectHandle>` fields
- Updated `load_level()` to allocate persons from pool and insert into cell grid alongside the existing Vec
- Added `ObjectPool::clear()` to reinitialize all slots and free list
- Added `ObjectPool::slots()` / `slots_mut()` for CellGrid interop
- Added `CellGrid::clear()` to reset all cell heads
- Added `sync_units_from_pool()` compatibility bridge method
- Added `pool()`, `pool_mut()`, `cell_grid()` accessor methods

### Task 2: Migrate access sites to units() accessor (7d351be)

- Made `units` field private (was `pub units: Vec<Unit>`)
- Added `pub fn units(&self) -> &[Unit]` accessor
- Updated all 9 app.rs access sites to use `units()`
- Updated sprites/mod.rs selection outline access to use `units()`
- Updated coordinator test to use `units()` accessor

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed missed access site in sprites/mod.rs**
- **Found during:** Task 2
- **Issue:** Plan listed 9 app.rs sites but missed `coordinator.units.get(unit_id)` in `src/render/sprites/mod.rs:540`
- **Fix:** Updated to `coordinator.units().get(unit_id)`
- **Files modified:** src/render/sprites/mod.rs
- **Commit:** 7d351be

## Decisions Made

1. **Compatibility shim approach**: Kept Vec<Unit> as a derived view from pool rather than migrating tick() logic all at once. This is the lowest-risk path since tick() has complex multi-unit borrow patterns (combat detection reads multiple units simultaneously). The pool is source of truth for allocation; the Vec stays current via sync.

2. **Private field + accessor**: Making `units` private and exposing `units() -> &[Unit]` enforces that all external code goes through the accessor, preparing for future full migration where the accessor could return pool-backed iterators instead.

## Verification

- `cargo test`: 289 tests pass (all existing tests preserved)
- `cargo build`: compiles with no errors
- No direct `.units` field access from outside coordinator.rs
- Pool populated during load_level with correct cell grid insertion

## Self-Check: PASSED

- FOUND: c628c80 (Task 1 commit)
- FOUND: 7d351be (Task 2 commit)
- FOUND: 01-03-SUMMARY.md
- All 289 tests passing
