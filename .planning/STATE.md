---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: in_progress
stopped_at: Completed 03-01-PLAN.md (string table + font data parsers)
last_updated: "2026-03-18T15:25:00Z"
last_activity: 2026-03-18 -- Completed 03-01 String table and font data parsers
progress:
  total_phases: 5
  completed_phases: 2
  total_plans: 18
  completed_plans: 17
  percent: 94
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-17)

**Core value:** Faithful reproduction of the original Populous: The Beginning gameplay on modern platforms
**Current focus:** Phase 3: HUD and Effects

## Current Position

Phase: 3 of 5 (HUD and Effects) -- IN PROGRESS
Plan: 4 of 5 in current phase (03-01 done, 03-02 done, 03-03 done)
Status: Executing Phase 3 plans
Last activity: 2026-03-18 -- Completed 03-01 String table and font data parsers

Progress: [█████████░] 94%

## Performance Metrics

**Velocity:**
- Total plans completed: 7
- Average duration: ~4 min
- Total execution time: ~27 min

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-core-object-system | 3/3 | ~7 min | ~2.3 min |
| 02-economy-and-combat | 7/7 | ~32 min | ~4.6 min |

**Recent Trend:**
- Last 5 plans: 02-03 (4min), 02-01 (4min), 02-05 (5min), 02-06 (7min), 02-07 (5min)
- Trend: Fast

*Updated after each plan completion*
| Phase 02 P06 | 7min | 2 tasks | 8 files |
| Phase 02 P07 | 5min | 2 tasks | 8 files |
| Phase 02 P10 | 5min | 2 tasks | 4 files |
| Phase 02 P09 | 7min | 2 tasks | 5 files |
| Phase 03 P02 | 3min | 2 tasks | 4 files |
| Phase 03 P03 | 4min | 2 tasks | 2 files |
| Phase 03 P01 | 4min | 2 tasks | 4 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- [Roadmap]: Lua scripting for AI instead of bytecode VM (community docs exist for Lua equivalents)
- [Roadmap]: Coarse 4-phase structure consolidating research's 9 phases into delivery boundaries
- [Roadmap]: Audio, vehicles, creatures, remaining spells deferred to v2
- [01-01]: Single LIFO free list instead of original binary's two-tier high/low priority split
- [01-01]: Box<[PoolSlot]> via Vec for heap allocation without stack overflow
- [01-02]: CellGrid kept separate from RegionMap to avoid repr(C) layout issues
- [01-02]: Reused REGION_GRID_SIZE constant from movement module for CELL_GRID_SIZE
- [01-03]: Kept Vec<Unit> as compatibility shim rebuilt from pool, avoiding risky all-at-once tick() migration
- [01-03]: Made units field private with pub fn units() accessor for encapsulation
- [02-03]: Person subtype mapping: Wild(1)=0 mana, Brave(2)=1, Warrior(3)=1, Preacher(4)=2, Spy(5)=1, SuperWarrior(6)=1, Shaman(7)=1
- [02-03]: u16 for population/wood types, u32 for mana (matching MAX_MANA=1000000 range)
- [02-01]: BuildingSubtype gaps match original binary (no type 12, jumps 11->13)
- [02-01]: Behavior flags from BLD.5: 0x20=housing, 0x01=training, 0x40=vehicle, 0x08=fighting, 0x0400=temple
- [Phase 02]: Normal calculation via cross product of tangent vectors T_z x T_x for correct up-facing normals on flat terrain
- [Phase 02]: Steep slope threshold at 512 height units between adjacent cells
- [Phase 02]: CascadeRegion handles toroidal wrapping internally with for_each_cell and contains_tile
- [02-05]: Reuse construction_progress as spawn timer in Active state (matches original binary pattern)
- [02-05]: Building combat base damage = 100 per fighter slot per tick
- [02-05]: Placement checks water (0x04), steep (0x02), occupied in priority order
- [02-06]: ShotData fields match original binary projectile tracking (type, target, damage, AOE, knockback, lifetime, speed)
- [02-06]: fight_damage_for_subtype constants cross-verified against person_type_defaults table
- [02-06]: Drum tower range 768 world units (about 6 cells at 128 units/cell)
- [02-04]: DeferredAction pattern avoids borrow checker conflicts between Unit tick and building pool data
- [02-04]: tick_state returns (TickResult, DeferredAction) tuple; existing states return DeferredAction::None
- [02-04]: Guard behavior uses guard_position field on Unit, managed by coordinator (no Guard enum variant)
- [02-07]: ManaTickBridge pattern: separate struct holding pool ref + tribe data ref to bridge borrow-checker constraint
- [02-07]: Mana tick called post-simulation_tick outside TickSubsystems due to borrow conflict with coordinator in objects slot
- [02-07]: Ghost preview rendering placeholder with alpha/tint logic; full GPU uniform integration deferred to render pipeline refactor
- [02-10]: Default tribe_index=0 (Blue) for ghost preview; player tribe selection is follow-up
- [02-10]: Ghost mesh cached by (building_type, cell_x, cell_y) key to avoid per-frame GPU rebuild
- [02-10]: Ghost pipeline depth_write_enabled=false so transparent ghost doesn't occlude objects
- [02-08]: BuildingTickActions struct aggregates spawn/convert/combat from single building tick
- [02-08]: Two-phase collect-then-process in tick_buildings() avoids borrow conflicts
- [02-08]: spawn_brave_near offsets spawn position by (128, 64) world units from building
- [02-09]: gather_target field on Unit instead of reusing movement.target_pos (avoids pathfinding conflicts)
- [02-09]: state_timer as flag (0=need target, 1=navigating) for Gathering state machine
- [02-09]: AOE radius to cell radius via (radius / 128).max(1) for CellGrid knockback queries
- [03-01]: Integer scaling of 8x8 base font rather than loading original .fon files (sufficient quality, simpler)
- [03-01]: draw_text_sized delegates to atlas-based draw_text with computed pixel size (avoids duplicate render path)
- [03-02]: LIFO free list with Vec<u16> for EffectPool (same pattern as ObjectPool, cache-friendly)
- [03-02]: Effect state=0xFF as inactive sentinel matching original binary pattern
- [03-02]: Two-phase entity position sync via EntityPosition struct (matches DeferredAction pattern)
- [03-03]: Mana displayed in K units (player_mana / 1000) for readability
- [03-03]: spell_cooldowns as Vec<SpellCooldown> populated empty now, Phase 4 fills from SpellSystem
- [03-03]: Population display placed below mana bar in sidebar layout

### Pending Todos

None yet.

### Blockers/Concerns

- Phase 1 (Object Pool): RESOLVED -- UnitCoordinator migration completed, all 289 tests pass.
- Phase 4 (AI): Lua scripting approach needs validation against community script documentation.
- app.rs is 3296 lines -- may need decomposition before or during Phase 2/3 render work.

## Session Continuity

Last session: 2026-03-18T15:24:00Z
Stopped at: Completed 03-03-PLAN.md (HudState mana, population, spell cooldowns)
Resume file: None
