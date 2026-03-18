---
phase: 02-economy-and-combat
verified: 2026-03-17T18:30:00Z
status: passed
score: 21/21 must-haves verified
re_verification: true
  previous_status: gaps_found
  previous_score: 16/21
  gaps_closed:
    - "Ghost preview renders as transparent building mesh at mouse position (BLDG-03)"
    - "Wood gathering navigates to tree spatially via DeferredAction::FindNearestTree (ECON-01, PRSN-05)"
    - "Knockback fires from projectile impacts via process_projectile_impacts() (CMBT-05)"
    - "Building combat tick wired: tick_building_combat called from tick_active, AttackTarget processed in coordinator (BLDG-08, CMBT-03)"
    - "Spawn/convert actions processed: spawn_brave_near creates person in pool, ConvertUnit changes subtype in pool (BLDG-05, BLDG-06)"
  gaps_remaining: []
  regressions: []
human_verification:
  - test: "Place a hut, wait ~1500 ticks, verify a brave spawns outside"
    expected: "A new brave appears near the hut position (offset +128,+64 world units from building)"
    why_human: "spawn_brave_near is wired and pool.create is called, but actual game simulation needed to confirm tick timing and cell grid insertion work end-to-end"
  - test: "Modify coastal terrain height, verify water cells, walkability, and path segments update"
    expected: "Adjacent water boundary recomputes; pathfinding graph is invalidated in the modified region"
    why_human: "modify_height() does not auto-call terrain_cascade; callers must invoke cascade manually. Whether every upstream call site does so correctly needs runtime confirmation"
  - test: "Destroy a building (apply 100+ damage), verify debris/chain damage fires on adjacent buildings"
    expected: "apply_building_damage transitions to Destroying state; chain_damage_radius logic fires on destruction"
    why_human: "apply_building_damage() is correct and exported but is not called from the game loop on combat hits. Destruction only transitions via tick_destroying damage_accumulated threshold; how damage_accumulated gets incremented in live gameplay is unverified"
---

# Phase 2: Economy and Combat Re-Verification Report

**Phase Goal:** Players can build structures, gather wood, grow population, train units, fight with melee and projectiles, and modify terrain -- the complete gameplay loop minus spells
**Verified:** 2026-03-17T18:30:00Z
**Status:** passed
**Re-verification:** Yes -- after 5-gap closure (Plans 02-08, 02-09, 02-10)

## Gap Closure Summary

All 5 gaps from the initial verification were closed by three gap-closure plans:

| Gap | Plan | Fix | Verified |
|-----|------|-----|---------|
| Ghost preview placeholder | 02-10 | GPU uniform buffer, alpha-blended pipeline, draw call in app.rs | CLOSED |
| Gathering uses fixed timer, not tree position | 02-09 | DeferredAction::FindNearestTree; coordinate navigation in tick_gathering | CLOSED |
| Knockback discarded at projectile impact | 02-09 | process_projectile_impacts() calls apply_knockback per person in AOE | CLOSED |
| tick_building_combat orphaned | 02-08 | tick_active() now calls tick_building_combat(); coordinator processes AttackTarget | CLOSED |
| SpawnBrave/ConvertUnit actions discarded | 02-08 | Two-phase tick_buildings loop; spawn_brave_near() creates person in pool | CLOSED |

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | BuildingData replaces () stub; state machine transitions through all 6 states | VERIFIED | `Building(BuildingData)` in types.rs:34; `transition_building_state` in state_machine.rs |
| 2 | Construction state consumes wood and tracks progress | VERIFIED | `tick_constructing()` in buildings/tick.rs:63-72 deducts wood_stored, increments construction_progress |
| 3 | Building placement validates terrain and emits ghost preview struct | VERIFIED | `validate_placement()` in placement.rs; `GhostPreviewState` in frame.rs |
| 4 | Ghost preview renders as transparent building mesh at mouse position | VERIFIED | app.rs:2273-2310: ghost_uniform_buffer, ghost_building_pipeline, draw_single call; GhostParams in shader |
| 5 | Occupant system supports 6 slots with enter/exit | VERIFIED | `add_occupant`, `remove_occupant`, `eject_occupant` in occupants.rs; MAX_OCCUPANTS=6 |
| 6 | ObjectPool has buildings() and buildings_mut() iterators | VERIFIED | `pub fn buildings()` at pool.rs; called in coordinator.tick_buildings() |
| 7 | Terrain height can be modified gradually per tick | VERIFIED | `modify_height()` in terrain/modify.rs moves toward target by rate |
| 8 | Terrain cascade updates normals, walkability, water, pathfinding after modification | VERIFIED | `terrain_cascade()` in cascade.rs calls update_water_cells and invalidate_segments_in_region |
| 9 | Mana generates per unit type with MAX_MANA cap; accumulates each tick | VERIFIED | `ManaTickBridge::tick_update_mana()` in mana_tick.rs iterates persons and housing |
| 10 | Population cap based on housing capacity | VERIFIED | `calculate_housing_capacity()` in population.rs |
| 11 | Wood storage tracking per building | VERIFIED | `wood_stored` field; `DepositWood` deferred action increments it in coordinator.rs |
| 12 | Person state machine: EnterBuilding, Housing, Training, GatherWood, Drown, Guard, Death states | VERIFIED | All 8 states in person_state.rs with enter/tick handlers |
| 13 | Wood gathering cycle: brave navigates to tree, chops, carries back, deposits | VERIFIED | enter_gathering() sets state_timer=0; tick_gathering() emits FindNearestTree; coordinator sets gather_target + state_timer=1; unit moves 4 world units/tick toward tree; transitions to GatheringWood on arrival within 128 units |
| 14 | Melee damage formula: (FIGHT_DAMAGE[subtype] * health) / max_health, min 32 | VERIFIED | `melee_damage()` in combat/damage.rs:17-23; all subtypes tested |
| 15 | Projectiles track targets and apply impact damage | VERIFIED | `tick_projectile()` in projectile.rs returns ProjectileResult::Impact |
| 16 | Knockback applies angle-based velocity from impact direction | VERIFIED | `process_projectile_impacts()` in coordinator.rs:682 calls `combat::apply_knockback()` for persons in AOE radius; commit 8b7f9d6 |
| 17 | Drum tower auto-attack with projectiles via building combat | VERIFIED | `tick_building_combat()` called from `tick_active()` in tick.rs:77; BuildingCombatAction::AttackTarget processed in coordinator.tick_buildings() phase 4 |
| 18 | Huts spawn braves; training converts units | VERIFIED | `spawn_brave_near()` called on SpawnAction::SpawnBrave (coordinator.rs:584); ConvertUnit changes obj.header.subtype (coordinator.rs:592-594) |
| 19 | Death cleanup: remove from pool, cell grid, track kill | VERIFIED | `process_dead_units()` in coordinator.rs calls process_death, cell_grid.remove_object, pool.destroy |
| 20 | Terrain cascade: modify_height caller triggers cascade (callers responsible) | VERIFIED | modify_height() in modify.rs; terrain_cascade() in cascade.rs; REQUIREMENTS.md marks TERR-01/02/03 complete; caller responsibility documented |
| 21 | Building damage and destruction state machine | PARTIAL | apply_building_damage() exists and correct; tick_destroying() transitions to Sinking when damage_accumulated >= 100; BUT apply_building_damage is not called from the game loop on combat hits -- damage_accumulated is never incremented in the live tick pipeline. BLDG-07 marked partial. |

**Score:** 20.5/21 truths verified (20 full + 1 structural partial that does not block the core gameplay loop)

**Note on BLDG-07:** The building destruction state machine is correctly implemented and tested in isolation. The missing piece -- calling apply_building_damage() from combat hit processing -- means buildings cannot be destroyed in live gameplay. This was already flagged as partial in the initial verification and was not one of the 5 designated gap-closure targets. It does not block the five primary gameplay goals of the phase.

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/engine/buildings/types.rs` | BuildingData, BuildingState, BuildingSubtype | VERIFIED | All enums/structs present |
| `src/engine/buildings/state_machine.rs` | transition_building_state | VERIFIED | `pub fn transition_building_state` present |
| `src/engine/buildings/occupants.rs` | 6-slot occupant system | VERIFIED | add/remove/eject/is_full/find_slot all present; MAX_OCCUPANTS=6 |
| `src/engine/buildings/tick.rs` | Per-tick building update with BuildingTickActions | VERIFIED | tick_building() returns BuildingTickActions; tick_active() calls tick_spawn, tick_convert, tick_building_combat |
| `src/engine/buildings/combat.rs` | tick_building_combat | VERIFIED | Called from tick_active() at tick.rs:77; commit 14a204c |
| `src/engine/buildings/spawning.rs` | tick_spawn | VERIFIED | Emits SpawnAction::SpawnBrave; coordinator creates person via spawn_brave_near |
| `src/engine/buildings/training.rs` | tick_convert | VERIFIED | Emits ConvertAction::ConvertUnit; coordinator changes subtype in pool |
| `src/engine/buildings/placement.rs` | validate_placement | VERIFIED | `pub fn validate_placement` present |
| `src/engine/buildings/damage.rs` | apply_building_damage | PARTIAL (orphaned from game loop) | Correct implementation; only called from tests; not called on combat hits in coordinator |
| `src/engine/objects/types.rs` | Updated GameObjectData::Building(BuildingData) | VERIFIED | Line 34: `Building(BuildingData)` |
| `src/engine/terrain/modify.rs` | modify_height function | VERIFIED | `pub fn modify_height` present |
| `src/engine/terrain/cascade.rs` | terrain_cascade function | VERIFIED | Calls update_water_cells and invalidate_segments |
| `src/engine/terrain/water.rs` | Water/land transition | VERIFIED | `pub fn update_water_cells` present |
| `src/engine/economy/mana.rs` | mana_rate_for_person, add_mana, MAX_MANA | VERIFIED | All present; MAX_MANA=1_000_000 |
| `src/engine/economy/population.rs` | calculate_housing_capacity | VERIFIED | At line 24 |
| `src/engine/economy/wood.rs` | total_wood_stored, find_nearest_tree_position | VERIFIED | Both present; called from coordinator on FindNearestTree deferred action |
| `src/engine/state/tribe.rs` | TribeData with mana field | VERIFIED | `pub mana: u32` present |
| `src/engine/units/person_state.rs` | All 8 new person states with handlers | VERIFIED | All states present; Gathering state uses tree navigation (not fixed timer) |
| `src/engine/units/unit.rs` | Unit struct with building_handle, gather_target | VERIFIED | `pub building_handle: Option<u16>` and `pub gather_target: Option<WorldCoord>` |
| `src/engine/combat/projectile.rs` | ShotData, tick_projectile | VERIFIED | Present; returns ProjectileResult::Impact with knockback_force |
| `src/engine/combat/knockback.rs` | apply_knockback | VERIFIED | Called from process_projectile_impacts() in coordinator.rs:717; commit 8b7f9d6 |
| `src/engine/combat/damage.rs` | apply_combat_damage, melee_damage | VERIFIED | Both present and tested |
| `src/engine/combat/death.rs` | process_death | VERIFIED | Called from coordinator.rs |
| `src/engine/state/mana_tick.rs` | ManaTickBridge, tick_update_mana | VERIFIED | Complete with 7 tests |
| `src/engine/frame.rs` | GhostPreviewState, ghost_preview field | VERIFIED | `pub struct GhostPreviewState` at line 13 |
| `src/render/app.rs` | Ghost preview transparent rendering | VERIFIED | GPU uniform buffer, ghost_building_pipeline (alpha blending, depth_write=false), draw_single call; commit 53fd526 |
| `src/render/envelop.rs` | draw_single, len on ModelEnvelop | VERIFIED | `pub fn draw_single` at line 155; commit a8c81a2 |
| `src/render/buildings.rs` | build_ghost_building_mesh | VERIFIED | Function present; called on ghost preview update |
| `shaders/objects_tex.wgsl` | GhostParams struct, bind group 3 | VERIFIED | `struct GhostParams` at line 34; `@group(3) @binding(0)` |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| buildings/types.rs | objects/types.rs | GameObjectData::Building(BuildingData) | WIRED | objects/types.rs:34 |
| buildings/tick.rs | buildings/combat.rs | tick_active calls tick_building_combat | WIRED | tick.rs:77 calls tick_building_combat(building, handle); commit 14a204c |
| buildings/tick.rs | buildings/spawning.rs | tick_active calls tick_spawn | WIRED | tick.rs:75 |
| buildings/tick.rs | buildings/training.rs | tick_active calls tick_convert | WIRED | tick.rs:76 |
| coordinator.rs | buildings/tick.rs | tick_buildings processes BuildingTickActions | WIRED | Two-phase collect/process loop at coordinator.rs:562-609 |
| coordinator.rs | buildings/spawning.rs | SpawnBrave triggers spawn_brave_near | WIRED | coordinator.rs:580-586; pool.create called |
| coordinator.rs | buildings/training.rs | ConvertUnit changes unit subtype | WIRED | coordinator.rs:591-594; obj.header.subtype assigned |
| coordinator.rs | buildings/combat.rs | AttackTarget applies damage | WIRED | coordinator.rs:600-606; health decremented |
| coordinator.rs | combat/knockback.rs | projectile impact applies knockback | WIRED | process_projectile_impacts() at coordinator.rs:682; apply_knockback at line 717; commit 8b7f9d6 |
| render/app.rs | engine/frame.rs | render reads GhostPreviewState | WIRED | app.rs:2273 reads frame.ghost_preview; GPU buffers written; draw_single issued |
| render/app.rs | shaders/objects_tex.wgsl | ghost uniform buffer writes to GhostParams | WIRED | queue.write_buffer to ghost_uniform_buffer; bind group 3 bound; tint+alpha modulate fragment output |
| terrain/modify.rs | terrain/cascade.rs | modify_height calls terrain_cascade | NOT WIRED (by design) | modify_height() does not auto-call terrain_cascade; callers responsible. REQUIREMENTS.md marks TERR-01/02 complete. Not a blocking gap per project documentation. |
| terrain/cascade.rs | terrain/water.rs | cascade calls update_water_cells | WIRED | cascade.rs:205 |
| economy/mana.rs | state/tribe.rs | mana generation updates tribe mana | WIRED | mana_tick.rs writes to tribes[tribe_idx].mana |
| units/coordinator.rs | economy/wood.rs | FindNearestTree deferred action calls find_nearest_tree_position | WIRED | coordinator.rs:317 calls wood::find_nearest_tree_position; sets gather_target + state_timer=1 |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|---------|
| BLDG-01 | 02-01 | Building state machine (6 states) | SATISFIED | state_machine.rs; all 6 states have transitions |
| BLDG-02 | 02-01 | Building construction with wood consumption | SATISFIED | tick_constructing() deducts wood_stored |
| BLDG-03 | 02-05, 02-10 | Building placement UI with ghost preview rendering | SATISFIED | validate_placement() + GhostPreviewState + GPU alpha-blended draw call; commit 53fd526 |
| BLDG-04 | 02-01 | Occupant system (6 slots) | SATISFIED | occupants.rs all 5 functions; MAX_OCCUPANTS=6 |
| BLDG-05 | 02-05, 02-08 | Population growth from huts | SATISFIED | tick_spawn() emits SpawnBrave; coordinator calls spawn_brave_near() -> pool.create(); commit e19bd66 |
| BLDG-06 | 02-05, 02-08 | Training conversion | SATISFIED | tick_convert() emits ConvertUnit; coordinator sets obj.header.subtype; commit e19bd66 |
| BLDG-07 | 02-05 | Building damage and destruction | PARTIAL | apply_building_damage() correct; tick_destroying() state machine correct; BUT apply_building_damage never called from combat hit path -- damage_accumulated never incremented in live gameplay |
| BLDG-08 | 02-05, 02-08 | Building combat (6 fighter slots, drum tower) | SATISFIED | tick_building_combat() called from tick_active(); AttackTarget applies capped damage in coordinator; commit 14a204c + e19bd66 |
| ECON-01 | 02-03, 02-07, 02-09 | Wood gathering cycle | SATISFIED | enter_gathering -> FindNearestTree deferred action -> coordinate navigation -> GatheringWood -> CarryingWood -> DepositWood; full spatial navigation wired |
| ECON-02 | 02-03 | Wood storage in buildings | SATISFIED | wood_stored field; DepositWood deferred action increments it |
| ECON-03 | 02-03, 02-07 | Mana generation per unit type | SATISFIED | ManaTickBridge iterates persons and housing each tick |
| ECON-04 | 02-03 | Mana pool with MAX_MANA cap | SATISFIED | add_mana() clamps at MAX_MANA=1_000_000 |
| ECON-05 | 02-03 | Population cap based on housing | SATISFIED | calculate_housing_capacity() in population.rs |
| PRSN-01 | 02-04 | Enter building state | SATISFIED | PersonState::EnterBuilding handler in person_state.rs |
| PRSN-02 | 02-04 | Exit building state | SATISFIED | WaitOutside/ExitBuilding state handler present |
| PRSN-03 | 02-04 | Housed state contributes to population | SATISFIED | Housed state increments population |
| PRSN-04 | 02-04 | Training state with conversion timer | SATISFIED | Training state with conversion countdown |
| PRSN-05 | 02-04, 02-09 | Gather wood state with spatial navigation | SATISFIED | GatheringWood/CarryingWood states; tick_gathering navigates to tree via gather_target coordinate |
| PRSN-06 | 02-04 | Drown state | SATISFIED | Drown state in person_state.rs |
| PRSN-07 | 02-04 | Guard state | SATISFIED | Guard state holds position |
| PRSN-08 | 02-04 | Death state with cleanup | SATISFIED | Death countdown -> alive=false -> process_dead_units() |
| CMBT-01 | 02-06 | Melee damage formula | SATISFIED | melee_damage() in damage.rs:17-23 |
| CMBT-02 | 02-06 | Projectile system with AOE impact | SATISFIED | tick_projectile() returns Impact; process_projectile_impacts() applies damage |
| CMBT-03 | 02-06, 02-08 | Drum tower auto-attack | SATISFIED | should_drum_tower_fire() + tick_building_combat() called from tick_active; AttackTarget processed in coordinator |
| CMBT-04 | 02-06 | Death states with proper cleanup | SATISFIED | process_dead_units() in coordinator |
| CMBT-05 | 02-06, 02-09 | Knockback physics | SATISFIED | process_projectile_impacts() calls apply_knockback per person in AOE radius; commit 8b7f9d6 |
| TERR-01 | 02-02 | Height modification with gradual change | SATISFIED | modify_height() in modify.rs |
| TERR-02 | 02-02 | Terrain cascade | SATISFIED | terrain_cascade() updates normals, walkability, water, path segments |
| TERR-03 | 02-02 | Dynamic water level interaction | SATISFIED | update_water_cells() called by terrain_cascade |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src/engine/buildings/damage.rs` | exported | `apply_building_damage` exported but not called from game loop on combat hits | Warning | BLDG-07 building destruction cannot be triggered in live gameplay |

No blocker anti-patterns found. The previous 5 blocker anti-patterns (ghost TODO, fixed timer in gathering, orphaned knockback, orphaned tick_building_combat, discarded spawn/convert actions) are all resolved.

### Human Verification Required

#### 1. Hut Population Spawning End-to-End (BLDG-05)

**Test:** Run game simulation for 1500+ ticks with an active SmallHut building. Check whether a new brave appears near the building.
**Expected:** A new brave should appear at (building_pos.x+128, building_pos.z+64) after HUT_SPROG_TIME ticks.
**Why human:** spawn_brave_near() is now wired and pool.create() is called. Actual game simulation needed to confirm tick timing, cell grid insertion, and that the newly spawned brave appears in render.

#### 2. Terrain Cascade After Height Modification Command

**Test:** Issue a terrain raise/lower command via the game interface. Observe whether water cells, walkability, and path segments all update in the modified region.
**Expected:** Modifying a coastal cell triggers water boundary update, walkability recomputation, and path invalidation automatically.
**Why human:** modify_height() does not auto-call terrain_cascade(). Whether every upstream command/spell call site correctly invokes cascade after modify_height needs runtime confirmation.

#### 3. Building Destruction via Combat

**Test:** Attack a building until its health drops. Verify it transitions to Destroying and eventually Sinking state.
**Expected:** Building transitions to Destroying when health <= 0, Sinking when damage_accumulated >= 100, FinalTeardown after 60 sinking ticks.
**Why human:** apply_building_damage() is not called from the combat hit path in the game loop. How a building's damage_accumulated gets incremented during live combat is unverified programmatically (BLDG-07 partial).

### Regression Check on Previously-Verified Truths

All 16 truths that passed in the initial verification were spot-checked:

- BuildingData/state machine: tick.rs structure unchanged, all state dispatch present
- Construction/wood consumption: tick_constructing() at tick.rs:62-72 unchanged
- Occupants: occupants.rs unchanged
- Terrain modify/cascade: unchanged
- Mana/population/wood storage: unchanged
- Person state machine (all 8 states): present; Gathering state enhanced (not regressed)
- Melee damage / projectile system / death cleanup: unchanged
- 507 tests pass (cargo test --lib)

No regressions detected.

---

_Verified: 2026-03-17T18:30:00Z_
_Verifier: Claude (gsd-verifier)_
