---
phase: 02-economy-and-combat
plan: 05
subsystem: buildings
tags: [spawning, training, placement, damage, combat, game-commands]
dependency_graph:
  requires: [02-01, 02-03]
  provides: [building-behaviors, building-commands]
  affects: [tick-pipeline, game-commands]
tech_stack:
  patterns: [behavior-flags, action-enum, tdd]
key_files:
  created:
    - src/engine/buildings/spawning.rs
    - src/engine/buildings/training.rs
    - src/engine/buildings/placement.rs
    - src/engine/buildings/damage.rs
    - src/engine/buildings/combat.rs
  modified:
    - src/engine/buildings/mod.rs
    - src/engine/buildings/tick.rs
    - src/engine/command.rs
    - src/render/app.rs
decisions:
  - "Reuse construction_progress as spawn timer in Active state (matches original binary pattern)"
  - "Building combat base damage = 100 per fighter slot per tick"
  - "Placement checks water (0x04), steep (0x02), occupied in that priority order"
metrics:
  duration: ~10min
  completed: 2026-03-18
---

# Phase 2 Plan 5: Building Behaviors Summary

Hut spawning at 1500/1200/900 tick rates, training conversion with mana costs, placement validation, damage/destruction with wobble, and 6-slot building combat.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Building spawning and training systems | 1bf6679 | spawning.rs, training.rs, mod.rs, tick.rs |
| 2 | Building placement, damage, combat, GameCommand | 47adb7b | placement.rs, damage.rs, combat.rs, mod.rs, command.rs, app.rs |

## What Was Built

### Spawning (spawning.rs)
- `tick_spawn` checks behavior_flags & 0x20 and increments spawn timer
- HUT_SPROG_TIME rates: level 1 = 1500, level 2 = 1200, level 3 = 900 ticks
- Emits `SpawnAction::SpawnBrave` when threshold reached

### Training (training.rs)
- `tick_convert` counts down conversion timer and emits `ConvertAction::ConvertUnit`
- Conversion times: Warrior=256, Spy=192, Preacher=192, SuperWarrior=384
- Mana costs: Warrior=500, Spy=300, Preacher=400, SuperWarrior=800
- `start_training` sets countdown based on building subtype output

### Placement (placement.rs)
- `validate_placement` checks footprint cells against walkability and building flags
- Rejects water (bit 0x04), occupied cells, steep terrain (bit 0x02)
- Toroidal wrapping via `& 127` mask
- `GhostPreview` struct for placement UI

### Damage (damage.rs)
- `apply_building_damage` with 4-tick cooldown between hits
- Sets wobble (shake_x=8, shake_z=8) on damage
- Transitions to Destroying state at 0 HP
- Chain damage radius of 3 cells

### Combat (combat.rs)
- `tick_building_combat` iterates fighter slots and emits AttackTarget actions
- 6 max fighter slots, base damage 100 per attack
- `set_fighters` caps at occupant_count
- `set_building_target` sets/clears combat target

### GameCommand Extensions
- PlaceBuilding, CancelPlacement, EnterBuildMode, EnterBuilding, TrainUnit variants

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added GameCommand match arms in app.rs**
- **Found during:** Task 2
- **Issue:** New GameCommand variants caused non-exhaustive match error in render/app.rs
- **Fix:** Added catch-all arm returning false for building commands (no render effect)
- **Files modified:** src/render/app.rs
- **Commit:** 47adb7b

## Test Coverage

- 98 building tests (spawning, training, placement, damage, combat)
- 497 total tests passing
- All spawning rates, conversion times, mana costs verified
- Placement rejects water/occupied/steep, handles toroidal wrapping
- Damage cooldown, wobble, destruction at 0 HP tested
- Combat emits correct attacks per fighter slot

## Self-Check: PASSED

All 5 created files verified on disk. Both commits (1bf6679, 47adb7b) verified in git log.
