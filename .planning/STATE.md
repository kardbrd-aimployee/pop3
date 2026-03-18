---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: executing
stopped_at: Completed 02-06-PLAN.md (combat subsystem)
last_updated: "2026-03-18T03:37:45Z"
last_activity: 2026-03-18 -- Completed 02-06 Combat subsystem
progress:
  total_phases: 4
  completed_phases: 1
  total_plans: 10
  completed_plans: 6
  percent: 60
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-17)

**Core value:** Faithful reproduction of the original Populous: The Beginning gameplay on modern platforms
**Current focus:** Phase 2: Economy and Combat

## Current Position

Phase: 2 of 4 (Economy and Combat)
Plan: 7 of 7 in current phase (wave 2)
Status: In progress
Last activity: 2026-03-18 -- Completed 02-06 Combat subsystem

Progress: [######....] 60%

## Performance Metrics

**Velocity:**
- Total plans completed: 6
- Average duration: ~4 min
- Total execution time: ~22 min

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-core-object-system | 3/3 | ~7 min | ~2.3 min |
| 02-economy-and-combat | 6/7 | ~27 min | ~4.5 min |

**Recent Trend:**
- Last 5 plans: 01-03 (3min), 02-03 (4min), 02-01 (4min), 02-05 (5min), 02-06 (7min)
- Trend: Fast

*Updated after each plan completion*
| Phase 02 P06 | 7min | 2 tasks | 8 files |

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

### Pending Todos

None yet.

### Blockers/Concerns

- Phase 1 (Object Pool): RESOLVED -- UnitCoordinator migration completed, all 289 tests pass.
- Phase 4 (AI): Lua scripting approach needs validation against community script documentation.
- app.rs is 3296 lines -- may need decomposition before or during Phase 2/3 render work.

## Session Continuity

Last session: 2026-03-18T03:40:00Z
Stopped at: Completed 02-04-PLAN.md (person state extensions)
Resume file: None
