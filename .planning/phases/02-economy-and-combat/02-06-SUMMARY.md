---
phase: 02-economy-and-combat
plan: 06
subsystem: combat
tags: [combat, projectiles, knockback, damage, death, drum-tower]
dependency_graph:
  requires: [02-01]
  provides: [ShotData, tick_projectile, apply_knockback, apply_combat_damage, process_death, drum_tower_shot]
  affects: [03-spells, combat-tick-integration]
tech_stack:
  added: []
  patterns: [impact-result-enum, death-actions-struct, directional-knockback]
key_files:
  created:
    - src/engine/combat/mod.rs
    - src/engine/combat/projectile.rs
    - src/engine/combat/knockback.rs
    - src/engine/combat/damage.rs
    - src/engine/combat/death.rs
  modified:
    - src/engine/objects/types.rs
    - src/engine/objects/pool.rs
    - src/engine/mod.rs
decisions:
  - ShotData fields match original binary projectile tracking (type, target, damage, AOE, knockback, lifetime, speed)
  - fight_damage_for_subtype constants cross-verified against person_type_defaults table
  - Drum tower range 768 world units (about 6 cells at 128 units/cell)
metrics:
  duration: ~7 min
  completed: 2026-03-18
  tasks: 2/2
  tests_added: 29
  tests_total: 471
---

# Phase 2 Plan 6: Combat Subsystem Summary

Projectile tracking with ShotData in pool, angle-based knockback physics, unified damage application with original melee formula, death cleanup actions, and drum tower auto-attack targeting

## What Was Built

### Task 1: ShotData, Projectile Tracking, and Knockback Physics
- **ShotData** struct replaces `Shot(())` stub in ObjectPool with full fields: shot_type, target_handle, target_pos, damage, aoe_radius, knockback_force, lifetime, speed, source_handle
- **tick_projectile** moves shots toward target_pos by speed per tick, returns Impact when within threshold distance, Expired when lifetime runs out
- **apply_knockback** computes angle from impact point to target, applies directional velocity (force * direction / distance)
- **decay_knockback** halves velocity each tick for friction convergence
- **shots()/shots_mut()** pool iterators follow same pattern as persons() and buildings()
- **drum_tower_shot** factory creates standard drum tower projectiles (damage=150, aoe=2, knockback=64, speed=48)
- Shot type constants: SHOT_STANDARD=1, SHOT_TRAIL=2, SHOT_FIREBALL=4

### Task 2: Damage Application, Death Cleanup, and Drum Tower Auto-Attack
- **apply_combat_damage** applies raw damage to ObjectHeader, returns true if killed (health reaches 0)
- **melee_damage** formula: `(FIGHT_DAMAGE[subtype] * health) / max_health`, minimum 32 -- matches original binary exactly
- **fight_damage_for_subtype** constants verified against person_type_defaults for all 8 subtypes + fallback
- **process_death** returns DeathActions struct (handle, tribe, last_attacker_tribe) for caller to execute cleanup
- **should_drum_tower_fire** checks enemy tribe and distance within DRUM_TOWER_RANGE (768 world units)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] WorldCoord uses i16 not u16**
- **Found during:** Task 1
- **Issue:** Plan code snippets used `u16` with `wrapping_add`/`wrapping_sub` for WorldCoord fields, but actual type uses `i16`
- **Fix:** Used standard i16 arithmetic (`+=`, `-`) instead of wrapping operations
- **Files modified:** src/engine/combat/projectile.rs, src/engine/combat/knockback.rs

## Verification

- `cargo test combat` -- 29 tests pass (projectile: 6, knockback: 6, damage: 10, death: 7)
- `cargo test` -- 471 total tests pass (all existing + 29 new)
- Melee damage cross-verified: fight_damage_for_subtype matches person_type_defaults for all subtypes

## Self-Check: PASSED
