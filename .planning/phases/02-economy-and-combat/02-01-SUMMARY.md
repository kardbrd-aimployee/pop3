---
phase: 02-economy-and-combat
plan: 01
subsystem: buildings
tags: [buildings, state-machine, occupants, pool-integration]
dependency_graph:
  requires: [01-01, 01-02]
  provides: [BuildingData, BuildingState, BuildingSubtype, building-iterators]
  affects: [02-05, 02-06, 02-07]
tech_stack:
  added: []
  patterns: [state-machine, occupant-slots, typed-pool-iterators]
key_files:
  created:
    - src/engine/buildings/mod.rs
    - src/engine/buildings/types.rs
    - src/engine/buildings/state_machine.rs
    - src/engine/buildings/occupants.rs
    - src/engine/buildings/tick.rs
  modified:
    - src/engine/objects/types.rs
    - src/engine/objects/pool.rs
    - src/engine/objects/mod.rs
    - src/engine/mod.rs
decisions:
  - BuildingSubtype gaps match original binary (no type 12, jumps 11->13)
  - Behavior flags from BLD.5 -- 0x20=housing, 0x01=training, 0x40=vehicle, 0x08=fighting, 0x0400=temple
metrics:
  duration: ~4 min
  completed: 2026-03-18
  tasks: 2/2
  tests_added: 42
  tests_total: 399
---

# Phase 2 Plan 1: Building Data Foundation Summary

BuildingData struct with 6-state machine, 17 subtypes, 6-slot occupant system, construction tick, and ObjectPool integration replacing the () stub

## What Was Built

### Task 1: Building types, state machine, occupants (TDD)
Created `src/engine/buildings/` module with four files:
- **types.rs**: BuildingData struct (15 fields), BuildingState enum (6 states with repr(u8)), BuildingSubtype enum (17 types matching original binary), behavior flags and max health lookup functions
- **state_machine.rs**: Validated transitions (Init->ConstructionDone->Active->Destroying->Sinking->FinalTeardown), on_construction_complete sets behavior_flags, on_destroy clears occupants
- **occupants.rs**: 6-slot system with add/remove/eject/is_full/find operations
- **tick.rs**: Per-tick pipeline (damage cooldown, wobble decay, state dispatch), construction consuming wood, destroying/sinking transitions, construction_target per subtype

### Task 2: Pool integration
- Replaced `Building(())` with `Building(BuildingData)` in GameObjectData enum
- Updated ObjectPool::create() to initialize BuildingData::default()
- Added buildings() and buildings_mut() iterators matching persons() pattern
- Re-exported BuildingData from objects module

## Deviations from Plan

None - plan executed exactly as written.

## Verification

- `cargo test` passes with 399 tests (all 289+ existing tests still pass)
- `cargo test buildings` passes with 42 building-specific tests
- GameObjectData::Building(BuildingData) replaces the Building(()) stub
- ObjectPool::buildings() and buildings_mut() iterate building objects

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 | 13afd4a | Building types, state machine, occupant system (42 tests) |
| 2 | 5b21308 | Pool integration with building iterators |
