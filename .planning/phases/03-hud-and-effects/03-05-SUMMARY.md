---
phase: 03-hud-and-effects
plan: 05
subsystem: hud-effects
tags: [health-bars, effects, combat-fx, building-fx, spell-impact, world-projection]

# Dependency graph
requires:
  - phase: 03-hud-and-effects/02
    provides: "EffectPool with spawn/destroy/update_all, EffectType enum, spawn_at helper"
  - phase: 03-hud-and-effects/03
    provides: "HudState data contract with mana, population, spell cooldowns"
  - phase: 03-hud-and-effects/04
    provides: "unit_screen_pos, unit_pvm for world-to-screen projection"
provides:
  - "HealthBarEntry struct for world-projected health bars in HUD overlay"
  - "EffectAction deferred action enum for borrow-safe effect spawning"
  - "spawn_on_spell_impact() hook mapping all 12 spells to visual effects"
  - "Combat effect wiring: death puff, hit spark, blood spray"
  - "Building effect wiring: construction dust, destruction collapse, building fire"
  - "EffectPool integrated into game loop with per-tick update_all()"
affects: [04-ai-and-scripting]

# Tech tracking
tech-stack:
  added: []
  patterns: [deferred-effect-actions, world-to-screen-projection-for-hud, collect-then-process-effects]

key-files:
  created: []
  modified:
    - src/render/hud/mod.rs
    - src/render/app.rs
    - src/engine/effects/mod.rs
    - src/engine/effects/spawn.rs
    - src/engine/units/coordinator.rs

key-decisions:
  - "Health bars use existing unit_screen_pos/unit_pvm for world-to-screen projection (reuse, no new matrix computation)"
  - "EffectAction deferred pattern matches DeferredAction/BuildingTickActions collect-then-process approach"
  - "Effect actions collected in pending_effect_actions Vec on UnitCoordinator, drained by app loop"
  - "WorldCoord i16 fields cast to i32 for EffectAction position (effects use wider coordinate range)"
  - "Building fire effect spawned every tick while in Destroying state with damage (continuous visual)"

patterns-established:
  - "drain_effect_actions() on UnitCoordinator for post-tick effect processing"
  - "HealthBarEntry in HudState for world-projected HUD elements"

requirements-completed: [HUD-06, FX-02, FX-03, FX-04]

# Metrics
duration: 7min
completed: "2026-03-18T15:43:27Z"
tasks_completed: 2
tasks_total: 2
files_modified: 5
tests_added: 8
tests_total_pass: 558
---

# Phase 3 Plan 5: Health Bars and Effect Spawning Wiring Summary

World-projected health bars above damaged units using unit_pvm matrix, plus EffectPool integration into game loop with combat/death/building effect spawning and tested spawn_on_spell_impact hook for all 12 spells.

## What Was Done

### Task 1: Health bars above damaged entities (HUD-06)
- Added `HealthBarEntry` struct and `HealthBarType` enum to HUD data model
- Projected damaged unit positions to screen space using existing `unit_pvm` and `unit_screen_pos` methods
- Rendered color-coded health bars (green > 50%, yellow > 25%, red <= 25%) in HUD overlay pass
- Only shown for alive units with health < max_health, filtered to on-screen positions
- Added 3 unit tests for HealthBarEntry and HealthBarType
- **Commit:** 8dbb07f

### Task 2: Effect spawning wiring and spell impact hook (FX-02, FX-03, FX-04)
- Added `EffectAction::SpawnAt` deferred action enum to effects module
- Added `spawn_on_spell_impact()` mapping all 12 spell types (0x01-0x0C) to visual effects
- Added `EffectPool` field to `GameEngine` with initialization and `update_all()` in tick loop
- Wired combat effects: HitSpark on melee strike, BloodSpray on fatal hit, DeathPuff on unit death
- Wired building effects: ConstructionDust on construction complete, DestructionCollapse on destroy transition, BuildingFire while destroying
- Added `drain_effect_actions()` to UnitCoordinator for collect-then-process pattern
- Added 5 unit tests for spawn_on_spell_impact (individual spells, unknown spell, all 12 coverage)
- **Commit:** 648d823

## Deviations from Plan

None - plan executed exactly as written.

## Decisions Made

1. **Reused unit_pvm for health bar projection** - No new MVP computation needed; existing unit_screen_pos already handles coordinate transforms including curvature.
2. **EffectAction collect-then-process pattern** - Matches established DeferredAction and BuildingTickActions patterns to avoid borrow checker conflicts.
3. **WorldCoord i16 to i32 cast** - Effects use wider i32 coordinates; WorldCoord positions are i16. Simple `as i32` casts at collection point.
4. **Building fire spawns every tick while Destroying** - Creates continuous fire visual rather than one-shot. BuildingFire effect type has LOOP flag in defaults.

## Verification

- `cargo build` compiles cleanly (warnings only, pre-existing)
- `cargo test --lib` passes all 558 tests
- `cargo test engine::effects` passes all 20 effect tests including 5 new spawn_on_spell_impact tests
- All acceptance criteria met for both tasks

## Self-Check: PASSED
