---
phase: 02-economy-and-combat
plan: 04
subsystem: person-states
tags: [person-state, building-occupancy, wood-gathering, deferred-action, state-machine]
dependency_graph:
  requires: [02-01]
  provides: [DeferredAction, building-states, wood-cycle-states, training-states]
  affects: [02-05, 02-06, 02-07]
tech_stack:
  added: []
  patterns: [deferred-action, timer-based-state-machine, tuple-return]
key_files:
  created: []
  modified:
    - src/engine/units/person_state.rs
    - src/engine/units/unit.rs
    - src/engine/units/coordinator.rs
    - src/engine/units/selection.rs
    - src/engine/objects/types.rs
decisions:
  - DeferredAction pattern avoids borrow checker conflicts between Unit tick and building pool data
  - Guard state not in PersonState enum -- guard_position field on Unit used by coordinator for GuardPost buildings
  - tick_state returns (TickResult, DeferredAction) tuple; existing states return DeferredAction::None
  - Training timer defaults to 256 ticks, preserves custom timer if pre-set before enter_state
metrics:
  duration: ~10 min
  completed: 2026-03-18
  tasks: 1/1
  tests_added: 12
  tests_total: 496
---

# Phase 2 Plan 4: Person State Machine Extensions Summary

DeferredAction pattern enabling building/resource interaction from person tick, with enter/tick handlers for 8 building/economy states and 12 new tests

## What Was Built

### Task 1: Building/economy state handlers with DeferredAction (TDD)

**Unit struct extensions** (unit.rs):
- `building_handle: Option<u16>` -- which building this person is associated with
- `wood_carried: u16` -- wood being carried during gathering cycle
- `guard_position: Option<WorldCoord>` -- position to hold for guard behavior

**DeferredAction enum** (person_state.rs):
- `None`, `AddToBuilding`, `RemoveFromBuilding`, `DepositWood`, `SpawnAtBuilding`
- Enables tick_state to signal building interactions without accessing the building pool

**New state handlers** (person_state.rs):
- **EnterBuilding**: Timer-based walk (30 ticks), transitions to InsideBuilding with AddToBuilding action
- **InsideBuilding**: Holds until coordinator transitions to Housing or Training based on building type
- **Housing**: Stays indefinitely (person contributes to population count)
- **Training/InTraining/InsideTraining**: 256-tick countdown, transitions to WaitOutside with SpawnAtBuilding action
- **WaitOutside**: Clears building_handle, 20-tick walk away, transitions to Idle
- **Gathering**: 40-tick walk to tree, transitions to GatheringWood
- **GatheringWood**: 60-tick chop timer, sets wood_carried=1, transitions to CarryingWood
- **CarryingWood**: 40-tick walk to building, deposits wood with DepositWood action, loops to Gathering

**Signature change**: `tick_state` now returns `(TickResult, DeferredAction)` instead of `TickResult`. All existing state handlers wrapped to return `DeferredAction::None`. Coordinator updated to destructure the tuple.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Guard state not in PersonState enum**
- **Found during:** Task 1 implementation
- **Issue:** Plan referenced "Guard" state but PersonState enum has no Guard variant (0x1C is BeingSacrificed)
- **Fix:** Added guard_position field to Unit for coordinator-level guard behavior at GuardPost buildings, skipped dedicated enter/tick handlers for non-existent enum variant
- **Files modified:** src/engine/units/unit.rs

**2. [Rule 3 - Blocking] PersonData struct needed matching fields**
- **Found during:** Task 1 implementation
- **Issue:** PersonData in objects/types.rs needed building_handle, wood_carried, guard_position fields for sync_units_from_pool
- **Fix:** Added fields to PersonData struct and its Default impl (already present from prior work)
- **Files modified:** src/engine/objects/types.rs

## Verification

- `cargo test person_state` -- 32 tests pass (20 existing + 12 new)
- `cargo test` -- 496 tests pass (all existing tests unbroken)
- All acceptance criteria met: DeferredAction enum, new Unit fields, all state handlers present

## Self-Check: PASSED
