---
phase: 02-economy-and-combat
plan: 09
subsystem: units/gathering-knockback-wiring
tags: [gathering, knockback, deferred-action, projectile, wiring, gap-closure]
dependency_graph:
  requires: []
  provides: [gathering-navigation, projectile-knockback, projectile-aoe-damage]
  affects: [person_state, coordinator, unit, types]
tech_stack:
  added: []
  patterns: [deferred-action-for-tree-lookup, spatial-navigation-gathering, cell-grid-aoe-query]
key_files:
  created: []
  modified:
    - src/engine/units/person_state.rs
    - src/engine/units/coordinator.rs
    - src/engine/units/unit.rs
    - src/engine/units/selection.rs
    - src/engine/objects/types.rs
decisions:
  - Added gather_target field to Unit and PersonData instead of reusing movement.target_pos (avoids pathfinding conflicts)
  - state_timer as flag (0=need target, 1=navigating) for Gathering state machine
  - Linear step movement (4 world units/tick per axis) for gathering navigation matching brave walk speed
  - Manhattan distance < 128 for tree arrival threshold (1 cell width)
  - AOE radius converted to cell radius via (radius / 128).max(1) for CellGrid queries
metrics:
  duration: 7min
  completed: "2026-03-18T04:35:00Z"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 5
  tests_passed: 507
---

# Phase 02 Plan 09: Gathering Navigation + Knockback Wiring Summary

DeferredAction-based tree navigation for wood gathering; projectile impact knockback and AOE damage wired into game loop.

## Task 1: Wire wood gathering to use DeferredAction for tree navigation

**Commit:** 53fd526 (bundled with concurrent 02-10 commit)

Previously, `enter_gathering()` set a fixed 40-tick timer and `tick_gathering()` transitioned to GatheringWood after 40 ticks regardless of tree proximity. Now:

- `enter_gathering()` sets `state_timer=0` (need tree target), clears `gather_target`
- `tick_gathering()` with `state_timer=0` emits `DeferredAction::FindNearestTree { unit_index }`
- Coordinator handles `FindNearestTree` by calling `find_nearest_tree_position()` from `economy::wood`, setting `gather_target` and `state_timer=1`
- `tick_gathering()` with `state_timer=1` moves unit 4 world units/tick toward tree, transitions to GatheringWood when Manhattan distance < 128

**Files modified:** person_state.rs, coordinator.rs, unit.rs, selection.rs, types.rs

## Task 2: Wire knockback from projectile impacts to nearby persons

**Commit:** 8b7f9d6

Previously, `tick_update_objects()` called `self.tick_projectiles()` with a semicolon, discarding the impact data. Now:

- `let impacts = self.tick_projectiles()` captures the return value
- New `process_projectile_impacts()` method iterates impacts, queries CellGrid for persons within AOE radius, applies `combat::apply_knockback()` for knockback force and subtracts AOE damage from health

**Files modified:** coordinator.rs

## Deviations from Plan

None - plan executed exactly as written.

## Verification

- All 507 tests pass (cargo test)
- `FindNearestTree` variant present in DeferredAction enum
- `find_nearest_tree_position` called from coordinator
- `apply_knockback` called from coordinator on projectile impacts
- No fixed 40-tick timer remains in `tick_gathering`
