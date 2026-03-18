# Phase 2: Economy and Combat - Context

**Gathered:** 2026-03-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Players can build structures, gather wood, grow population, train units, fight with melee and projectiles, and modify terrain — the complete gameplay loop minus spells. This is the largest phase (29 requirements across 5 subsystems: buildings, economy, person states, combat, terrain modification).

</domain>

<decisions>
## Implementation Decisions

### Building system approach
- Building state machine follows original binary's states (init, construction, active, destroying, sinking, teardown) but uses Rust enums, not raw state bytes
- Buildings are stored as objects in the ObjectPool (using the Building variant of GameObjectData)
- Building placement requires ghost preview (transparent rendering at mouse position) and terrain validation (footprint check, clearance)
- Construction consumes wood over time — braves must deliver wood to construction site
- Hut population spawning follows original HUT_SPROG_TIME rates per hut level (1/2/3)
- Building occupant system: 6 slots per building, enter/exit tracking via person states

### Economy loop design
- Wood is the only resource — gathered from trees by braves
- Wood gathering is a person state cycle: walk to tree → chop (animation) → carry wood → walk to building → deposit
- Mana generation rates per unit type and activity follow original binary constants (MANA_F_BRAVE, MANA_F_WARR, etc.)
- Mana per housing level follows original: MANA_F_HUT_LEVEL_1/2/3
- Population cap tied to housing capacity (hut levels determine max occupants)
- Training conversion costs (wood + mana + time) follow original constants

### Combat faithfulness
- Melee damage formula: `damage = (FIGHT_DAMAGE[subtype] * health) / max_health`, min 32 — match original exactly
- Projectile system for drum towers: shot tracking to target, AOE impact damage, knockback physics
- Knockback uses angle-based velocity from Combat_ApplyKnockback
- Death states: proper cleanup, kill tracking, remove from pool and cell grid
- Building combat: 6 fighter slots per building, occupant fighting with attack selection

### Person state machine extensions
- Add states matching original binary: EnterBuilding, ExitBuilding, Housed, Training, GatherWood, Drown, Guard, Death
- Each new state integrates with existing PersonState enum (already has repr(u8))
- State transitions follow original binary's state machine (documented in things-to-implement.md §2)
- Drowning detection already partially exists in coordinator tick — extend with proper death handling

### Terrain modification
- Terrain_ModifyHeight: gradual height change matching original function
- Full cascade after modification: heights → normals → walkability → buildings → water → pathfinding → mesh rebuild
- Dynamic water interaction: cells become water/land based on height vs water level
- This cascade function is reused by spells in Phase 3

### Claude's Discretion
- Exact plan decomposition (how to split 29 requirements into parallel-friendly plans)
- Whether to decompose app.rs before adding building rendering, or add incrementally
- Test strategy for game mechanic faithfulness (fixture-based vs behavioral)
- Order of implementation within waves (which subsystems to parallelize)

</decisions>

<specifics>
## Specific Ideas

- Original binary constants for economy/combat are extensively documented in `things-to-implement.md` with exact addresses and values — use these as ground truth
- Building footprint data (SHAPES.DAT) parsing is already DONE — reuse existing `ShapeFootprints` in `src/data/objects.rs`
- Terrain flattening for building placement is already DONE — `flatten_building_footprint()` and `smooth_terrain_area()` exist
- The compatibility shim from Phase 1 (Vec<Unit> rebuilt from pool) means rendering code works unchanged — new building rendering can follow the same pattern initially

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Building system
- `things-to-implement.md` §3 "Building System" — 19 building subtypes, state machine, behaviors, occupant system, placement, damage/destruction
- `docs/specs/` — Check for building-specific RE specs

### Economy
- `things-to-implement.md` §21 "Economy & Resources" — Wood gathering, building costs
- `things-to-implement.md` §22 "Population & Mana" — Mana generation rates, population mechanics

### Combat
- `things-to-implement.md` §5 "Combat System" — Melee damage formula, projectiles, knockback, building combat, victory conditions
- `things-to-implement.md` §2 "Person / Unit System" — Person state machine (44+ states), conversion, selection

### Terrain
- `things-to-implement.md` §7 "Terrain System" — Height modification, cell flags, water level

### Existing code (must understand)
- `src/engine/objects/pool.rs` — ObjectPool from Phase 1 (building objects go here)
- `src/engine/objects/cell_grid.rs` — CellGrid from Phase 1 (building spatial tracking)
- `src/engine/units/coordinator.rs` — UnitCoordinator with pool integration
- `src/engine/units/person_state.rs` — Existing PersonState enum
- `src/engine/movement/region.rs` — RegionMap walkability (buildings affect walkability)
- `src/data/objects.rs` — ShapeFootprints, building footprint data (already parsed)
- `src/render/app.rs` — Render loop, building mesh rendering (already DONE for static meshes)
- `src/render/buildings.rs` — Building mesh construction (already DONE)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ShapeFootprints` (src/data/objects.rs): SHAPES.DAT already parsed, 95 entries with footprint masks
- `flatten_building_footprint()` and `smooth_terrain_area()`: Already implemented for building placement
- `build_building_meshes()` (src/render/buildings.rs): 3D building mesh rendering already works
- `PersonState` enum (src/engine/units/person_state.rs): Has repr(u8) with many states already defined but not all implemented
- `ObjectPool` + `CellGrid` (src/engine/objects/): Phase 1 output, ready for building objects
- `RegionMap` (src/engine/movement/region.rs): Has `has_building()` flag per cell
- `GameWorld` tick subsystem traits (src/engine/state/traits.rs): Building tick can plug in here

### Established Patterns
- Tick subsystem traits: New subsystems implement a tick trait and wire into GameWorld's tick loop
- GameCommand for input: Building placement commands go through GameCommand enum
- FrameState for rendering: New building state info exposed via FrameState
- Vec<Unit> compatibility shim: Can use similar pattern for building iteration initially

### Integration Points
- `GameWorld::simulation_tick()` — Wire building tick, combat tick into existing subsystem order
- `UnitCoordinator::tick()` — Person state machine extensions (enter/exit building, gather wood, training)
- `RegionMap` — Update walkability when buildings are placed/destroyed
- `App::render()` — Building ghost preview rendering, health bars
- `GameEngine::apply_command()` — Building placement commands

</code_context>

<deferred>
## Deferred Ideas

- Spell integration with combat (Phase 3)
- AI building placement decisions (Phase 4)
- Vehicle production from boat/air huts (v2)
- Building fire damage and spread (can be simplified or deferred)
- Fog of war on minimap (Phase 3 HUD work)

</deferred>

---

*Phase: 02-economy-and-combat*
*Context gathered: 2026-03-18*
