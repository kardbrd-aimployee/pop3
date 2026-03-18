---
phase: 02-economy-and-combat
plan: 03
subsystem: economy
tags: [mana, population, wood, economy, tdd]
dependency_graph:
  requires: []
  provides: [economy-module, mana-generation, population-capacity, wood-costs, tribe-economy-fields]
  affects: [building-spawning, person-training, combat-mana-costs]
tech_stack:
  added: []
  patterns: [pure-calculation-module, match-dispatch, cap-at-max]
key_files:
  created:
    - src/engine/economy/mod.rs
    - src/engine/economy/mana.rs
    - src/engine/economy/population.rs
    - src/engine/economy/wood.rs
  modified:
    - src/engine/state/tribe.rs
    - src/engine/mod.rs
decisions:
  - "Person subtype mapping: Wild(1)=0 mana, Brave(2)=1, Warrior(3)=1, Preacher(4)=2, Spy(5)=1, SuperWarrior(6)=1, Shaman(7)=1"
  - "u16 for population/wood types, u32 for mana (matching MAX_MANA=1000000 range)"
metrics:
  duration: "~4 min"
  completed: "2026-03-18"
  tasks_completed: 2
  tasks_total: 2
  tests_added: 47
---

# Phase 2 Plan 3: Economy Module Summary

Economy data structures and pure calculations for mana generation, population capacity, and wood costs -- faithfully matching original binary constants from things-to-implement.md section 22.

## What Was Built

### Task 1: Economy module with mana, population, and wood tracking (TDD)

**Mana (mana.rs):** Per-unit generation rates (BRAVE=1, WARR=1, SPY=1, PREACH=2, SWARR=1, SHAMEN=1), per-housing rates (level 1/2/3 = 1/2/3), MAX_MANA=1000000 cap. Functions: mana_rate_for_person(), mana_rate_for_housing(), add_mana() with cap, deduct_mana() with insufficient check.

**Population (population.rs):** Housing capacity per hut level (3/4/5), MAX_POP_VALUE=199 tribe cap. Functions: hut_capacity(), calculate_housing_capacity() with cap, can_spawn().

**Wood (wood.rs):** Construction costs (small hut=3, medium=5, large=7, drum tower=5, temple=6, training=5, default=4). Training costs (warrior=3, preacher=2, spy=2, super warrior=5). Functions: construction_wood_cost(), training_wood_cost(), total_wood_stored().

**mod.rs:** Re-exports all public functions and key constants.

47 tests cover all constants, function dispatch, edge cases (caps, zero, unknown subtypes).

### Task 2: Extend TribeData with economy fields

Added to TribeData: `mana: u32`, `max_population: u16`, `wood_gathered: u32`. All initialized to 0 in new(). Economy module registered in engine/mod.rs.

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 (RED) | 4f90a3c | Failing tests for economy module |
| 1 (GREEN) | e5f7524 | Implement economy module (47 tests pass) |
| 2 | 2bfee0d | Extend TribeData with economy fields |

## Deviations from Plan

None -- plan executed exactly as written.

## Notes

- Pre-existing compilation errors in pool.rs (BuildingData) and terrain/water.rs (for_each_cell) from parallel wave-1 plans prevent full crate compilation. These are NOT caused by this plan's changes and will resolve when those plans complete their work.
- The `pub mod economy` line in engine/mod.rs was already present by the time Task 2 ran (added during Task 1 for TDD compilation).
- `pub mod buildings` and `pub mod terrain` were added to engine/mod.rs by parallel wave-1 plans.
