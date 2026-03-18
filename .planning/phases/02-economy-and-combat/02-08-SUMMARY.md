---
phase: 02-economy-and-combat
plan: 08
subsystem: buildings/tick-wiring
tags: [buildings, spawn, convert, combat, game-loop]
dependency_graph:
  requires: []
  provides: [building-tick-actions, spawn-wiring, convert-wiring, combat-wiring]
  affects: [coordinator, buildings/tick]
tech_stack:
  added: []
  patterns: [two-phase-collect-then-process, action-aggregation-struct]
key_files:
  created: []
  modified:
    - src/engine/buildings/tick.rs
    - src/engine/units/coordinator.rs
decisions:
  - BuildingTickActions struct aggregates all three action types from a single building tick
  - Two-phase pattern in tick_buildings (collect actions, then process) avoids borrow conflicts
  - spawn_brave_near offsets spawn position by (128, 64) world units from building
metrics:
  duration: 6min
  completed: "2026-03-18T04:33:00Z"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 2
  tests_passed: 507
---

# Phase 2 Plan 8: Wire Building Tick Actions Summary

BuildingTickActions struct aggregates spawn/convert/combat from tick_building(); coordinator processes all three in two-phase collect-then-apply loop with spawn_brave_near helper.

## What Was Done

### Task 1: Wire tick_building_combat into tick_active and return all actions
- Added `BuildingTickActions` struct with spawn, convert, and combat fields
- Changed `tick_building()` signature to accept `ObjectHandle` and return `BuildingTickActions`
- `tick_active()` now calls `tick_building_combat()` alongside existing spawn/convert
- Non-Active states return `BuildingTickActions::none()` (empty actions)
- Updated all 13 existing tests for new signature, added 2 new tests
- **Commit:** 53fd526

### Task 2: Process BuildingTickActions in coordinator.tick_buildings()
- Rewrote `tick_buildings()` with two-phase pattern: collect actions first, process second
- Phase 2: SpawnAction::SpawnBrave triggers `spawn_brave_near()` to create new person
- Phase 3: ConvertAction::ConvertUnit changes target person's subtype in pool
- Phase 4: BuildingCombatAction::AttackTarget applies capped damage to target health
- Added `spawn_brave_near()` helper: allocates in pool, sets PersonData fields, inserts in cell grid, tracks handle
- **Commit:** e19bd66

## Deviations from Plan

None - plan executed exactly as written.

## Verification

- `cargo test --lib`: 507 passed, 0 failed
- All acceptance criteria grep patterns confirmed present
- `tick_building_combat` called from `tick_active` in tick.rs
- `SpawnAction::SpawnBrave`, `ConvertAction::ConvertUnit`, `BuildingCombatAction::AttackTarget` all processed in coordinator.rs
