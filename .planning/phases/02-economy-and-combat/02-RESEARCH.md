# Phase 2: Economy and Combat - Research

**Researched:** 2026-03-17
**Domain:** Game mechanics implementation (building system, economy loop, person states, combat, terrain modification) in Rust
**Confidence:** HIGH

## Summary

Phase 2 is the largest phase (29 requirements across 5 subsystems) and implements the complete gameplay loop minus spells. The codebase already has strong foundations from Phase 1: ObjectPool with Building variant (currently a unit stub `()`), CellGrid for spatial tracking, PersonState enum with all 44 states defined (but only 8 implemented), melee damage formula, and terrain flattening for building placement. The `things-to-implement.md` and `docs/specs/buildings.md` provide byte-level accuracy for every constant, offset, and state machine needed.

The key architectural challenge is that `GameObjectData::Building(())` is currently a unit type stub -- it needs to become a real `BuildingData` struct holding occupants, wood storage, construction progress, training state, and combat state. This mirrors how `PersonData` holds person-specific fields. The existing tick subsystem trait pattern (TerrainTick, ObjectTick, ManaTick, PopulationTick) provides clean integration points for new building tick, combat tick, and economy tick logic.

The second challenge is that `UnitCoordinator` currently owns the `ObjectPool` and `CellGrid` but only handles persons. Buildings need to be created in the same pool and tracked in the same grid, but their tick logic is fundamentally different (state machine, wood consumption, spawning, training). The coordinator needs either expansion or a sibling `BuildingCoordinator` that shares pool/grid access.

**Primary recommendation:** Implement subsystems bottom-up (data structures first, then state machines, then integration), using the existing tick trait pattern and ObjectPool. Split into 5-6 plans: building data + state machine, economy/wood system, person state extensions, combat enhancements, terrain modification, and integration/rendering.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Building state machine follows original binary's states (init, construction, active, destroying, sinking, teardown) but uses Rust enums, not raw state bytes
- Buildings are stored as objects in the ObjectPool (using the Building variant of GameObjectData)
- Building placement requires ghost preview (transparent rendering at mouse position) and terrain validation (footprint check, clearance)
- Construction consumes wood over time -- braves must deliver wood to construction site
- Hut population spawning follows original HUT_SPROG_TIME rates per hut level (1/2/3)
- Building occupant system: 6 slots per building, enter/exit tracking via person states
- Wood is the only resource -- gathered from trees by braves
- Wood gathering is a person state cycle: walk to tree -> chop (animation) -> carry wood -> walk to building -> deposit
- Mana generation rates per unit type and activity follow original binary constants (MANA_F_BRAVE, MANA_F_WARR, etc.)
- Mana per housing level follows original: MANA_F_HUT_LEVEL_1/2/3
- Population cap tied to housing capacity (hut levels determine max occupants)
- Training conversion costs (wood + mana + time) follow original constants
- Melee damage formula: `damage = (FIGHT_DAMAGE[subtype] * health) / max_health`, min 32 -- match original exactly
- Projectile system for drum towers: shot tracking to target, AOE impact damage, knockback physics
- Knockback uses angle-based velocity from Combat_ApplyKnockback
- Death states: proper cleanup, kill tracking, remove from pool and cell grid
- Building combat: 6 fighter slots per building, occupant fighting with attack selection
- Add states matching original binary: EnterBuilding, ExitBuilding, Housed, Training, GatherWood, Drown, Guard, Death
- Each new state integrates with existing PersonState enum (already has repr(u8))
- State transitions follow original binary's state machine (documented in things-to-implement.md section 2)
- Drowning detection already partially exists in coordinator tick -- extend with proper death handling
- Terrain_ModifyHeight: gradual height change matching original function
- Full cascade after modification: heights -> normals -> walkability -> buildings -> water -> pathfinding -> mesh rebuild
- Dynamic water interaction: cells become water/land based on height vs water level
- This cascade function is reused by spells in Phase 3

### Claude's Discretion
- Exact plan decomposition (how to split 29 requirements into parallel-friendly plans)
- Whether to decompose app.rs before adding building rendering, or add incrementally
- Test strategy for game mechanic faithfulness (fixture-based vs behavioral)
- Order of implementation within waves (which subsystems to parallelize)

### Deferred Ideas (OUT OF SCOPE)
- Spell integration with combat (Phase 3)
- AI building placement decisions (Phase 4)
- Vehicle production from boat/air huts (v2)
- Building fire damage and spread (can be simplified or deferred)
- Fog of war on minimap (Phase 3 HUD work)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| BLDG-01 | Building state machine (init, construction, active, destroying, sinking, teardown) | BuildingState enum + BuildingData struct in ObjectPool; state transitions from docs/specs/buildings.md BLD.4 |
| BLDG-02 | Building construction with progress animation and wood consumption | Building_UpdateConstructing (0x4322B0); wood_stored field at +0x63; construction progress timer |
| BLDG-03 | Building placement UI with ghost preview and terrain validation | GameCommand::PlaceBuilding; ghost flag 0x10 at +0x0E; ShapeFootprints already parsed; flatten_building_footprint() exists |
| BLDG-04 | Occupant system (6 slots per building, enter/exit, capacity checks) | 6 occupant slots at +0x86; occupant_count at +0xA6; Building_EjectPerson (0x432800) |
| BLDG-05 | Population growth from huts (spawning braves at HUT_SPROG_TIME rates) | Building_UpdateActive_TrainOrSpawn (0x430960); flag 0x20; 17 spawn rate bands |
| BLDG-06 | Training conversion (brave to warrior/spy/preacher/super warrior) | Building_UpdateActive_Convert (0x430EF0); flag 0x01; CONV_TIME constants; wood+mana costs |
| BLDG-07 | Building damage and destruction with debris and chain damage | Building_ApplyDamage (0x434570); Building_OnDestroy (0x433BB0); damage at +0x9E |
| BLDG-08 | Building combat (6 fighter slots, attack types, occupant fighting) | Building_ProcessFightingPersons (0x438610); 8 combat sub-states; attack selection PRNG |
| ECON-01 | Wood gathering (brave walks to tree, chops, carries wood back) | PersonState::Gathering (0x13), GatheringWood (0x15), CarryingWood (0x16); Person_StartWoodGathering (0x502f70) |
| ECON-02 | Wood storage in buildings with consumption tracking | wood_stored at +0x63; Building_UpdateWoodConsumption (0x430430) |
| ECON-03 | Mana generation per unit type and activity | Tick_UpdateMana (0x4aeac0); MANA_F_BRAVE/WARR/SPY/PREACH/SWARR/SHAMEN constants |
| ECON-04 | Mana pool per tribe with MAX_MANA cap | Per-tribe mana pool; MAX_MANA cap constant |
| ECON-05 | Population cap based on housing capacity (hut levels 1-3) | MAX_POP_VALUE_HUT_1/2/3; MAX_POP_VALUE tribe cap |
| PRSN-01 | Enter building state | PersonState::EnterBuilding (0x0A); walk into building, become occupant |
| PRSN-02 | Exit building state | PersonState::WaitOutside (0x0F); Building_EjectPerson; facing direction |
| PRSN-03 | Housed state (inside housing, contributes to population) | PersonState::Housing (0x11); InsideBuilding (0x0B); population count contribution |
| PRSN-04 | Training state (in training building, conversion timer) | PersonState::Training (0x10); InTraining (0x0E); conversion_countdown at +0xA0 |
| PRSN-05 | Gather wood state | PersonState::Gathering (0x13), GatheringWood (0x15), CarryingWood (0x16); tree targeting |
| PRSN-06 | Drown state | PersonState::Drowning (0x17); already partially implemented in tick_drowning(); extend with death effects |
| PRSN-07 | Guard state (hold position) | PersonState placeholder; guard position tracking; no movement on idle |
| PRSN-08 | Death effects (proper death state with cleanup) | PersonState::Dead (0x18); existing tick_dead(); add pool/grid removal, kill tracking |
| CMBT-01 | Complete melee damage formula | Already implemented in calculate_melee_damage(); verify constants match binary exactly |
| CMBT-02 | Projectile system (shot types, tracking, AOE impact, knockback) | Shot_ProcessImpact (0x4fb620); Shot_Update (0x458800); new Shot model type in pool |
| CMBT-03 | Drum tower auto-attack with projectiles | Building in ACTIVE state with drum tower subtype (4); target selection; shot creation |
| CMBT-04 | Death states with proper cleanup and kill tracking | Tribe_TrackKill (0x4b5000); last attacker at offset 0xB0; pool.destroy() + cell_grid removal |
| CMBT-05 | Knockback physics (angle-based velocity) | Combat_ApplyKnockback (0x4d7490); angle-based velocity applied to header.velocity |
| TERR-01 | Height modification function (Terrain_ModifyHeight) with gradual change | Terrain_ModifyHeight (0x4ea2e0); gradual per-tick height adjustment |
| TERR-02 | Terrain cascade after modification | heights -> normals -> walkability -> buildings -> water -> pathfinding -> mesh; GPU heights buffer update |
| TERR-03 | Dynamic water level interaction | Cells become water/land based on height vs water level; RegionMap walkability update |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust (stable) | 1.77+ | Language | Project already uses stable Rust |
| wgpu | (existing) | GPU rendering | Already integrated for terrain/building/sprite rendering |
| cgmath | (existing) | Math types | Already used for Vector2/3/4, Matrix4 throughout |
| log | (existing) | Logging | Already used for debug logging |

### Supporting
No new dependencies needed. This phase is pure game logic implementation using existing infrastructure.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Manual state machines | State machine crate (e.g., statig) | Overkill for simple enum-based FSMs matching binary behavior |
| ECS (specs/bevy_ecs) | Entity-Component-System | Project uses custom ObjectPool, not ECS; migration would be massive |

## Architecture Patterns

### Recommended Project Structure
```
src/engine/
  buildings/              # NEW: Building subsystem
    mod.rs               # BuildingData, BuildingState, BuildingSubtype enums
    state_machine.rs     # Building state transitions (init->construction->active->destroying->sinking->teardown)
    occupants.rs         # 6-slot occupant system, enter/exit
    training.rs          # Training conversion pipeline
    spawning.rs          # Hut population spawning
    placement.rs         # Placement validation, ghost preview logic
    combat.rs            # Building combat (6 fighter slots)
    tick.rs              # Per-tick building update (matches BLD.7 pipeline)
  economy/               # NEW: Economy subsystem
    mod.rs               # TribeEconomy struct (wood, mana, population per tribe)
    wood.rs              # Wood gathering state cycle, tree interaction
    mana.rs              # Mana generation per unit type/activity/housing
    population.rs        # Population tracking, housing capacity
  combat/                # NEW: Enhanced combat subsystem
    mod.rs               # Combat system orchestrator
    projectile.rs        # Shot tracking, AOE impact, knockback
    knockback.rs         # Angle-based velocity physics
    damage.rs            # Unified damage application (units + buildings)
  terrain/               # NEW: Terrain modification subsystem
    mod.rs               # Terrain_ModifyHeight, cascade function
    cascade.rs           # Full cascade: heights->normals->walkability->water->pathfinding->mesh
    water.rs             # Dynamic water level interaction
  units/
    person_state.rs      # EXTEND: Add enter/tick for EnterBuilding, ExitBuilding, Housed, Training, GatherWood, Guard states
  objects/
    types.rs             # MODIFY: Replace Building(()) with Building(BuildingData)
```

### Pattern 1: Building State Machine (Rust Enum)
**What:** Building states as Rust enum with explicit transitions, mirroring BLD.4
**When to use:** All building lifecycle management
**Example:**
```rust
// Source: docs/specs/buildings.md BLD.4, things-to-implement.md section 3
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BuildingState {
    Init            = 0x01,
    ConstructionDone = 0x02,  // triggers OnConstructionComplete
    Active          = 0x03,   // main operational state
    Destroying      = 0x04,   // OnDestroy, debris spawn
    Sinking         = 0x05,   // sinking into ground
    FinalTeardown   = 0x06,   // cleanup and removal
}
```

### Pattern 2: BuildingData Struct (Replaces Unit Stub)
**What:** Full building-specific data mirroring binary layout
**When to use:** Stored in GameObjectData::Building(BuildingData)
**Example:**
```rust
// Source: docs/specs/buildings.md BLD.2
pub struct BuildingData {
    pub state: BuildingState,
    pub building_type: BuildingSubtype,
    pub wood_stored: u16,           // +0x63
    pub occupant_slots: [Option<ObjectHandle>; 6],  // +0x86
    pub occupant_count: u8,         // +0xA6
    pub construction_progress: u16, // for construction state
    pub conversion_countdown: u16,  // +0xA0 for training buildings
    pub training_countdown: u16,    // +0xA4
    pub damage_accumulated: u16,    // +0x9E
    pub damage_cooldown: u8,        // +0xAB
    pub behavior_flags: u32,        // from type properties at 0x5A0050
    pub shake_x: i16,               // +0x6C wobble
    pub shake_z: i16,               // +0x6E wobble
    pub num_fighting: u8,           // +0x68
    pub target_person: Option<ObjectHandle>, // +0x72
}
```

### Pattern 3: TribeEconomy Struct (Per-Tribe State)
**What:** Central economy state per tribe
**When to use:** Mana, wood, population tracking
**Example:**
```rust
pub struct TribeEconomy {
    pub mana: u32,           // current mana pool
    pub max_mana: u32,       // MAX_MANA cap
    pub population: u16,     // current alive units
    pub max_population: u16, // based on housing capacity
    pub wood_total: u32,     // total wood across all buildings (for display)
}
```

### Pattern 4: Tick Pipeline (Building_Update BLD.7)
**What:** Per-tick building update following original's exact order
**When to use:** Every game tick for each active building
**Example:**
```rust
// Source: docs/specs/buildings.md BLD.7
pub fn tick_building(building: &mut BuildingData, header: &mut ObjectHeader, ...) {
    // 1. Footprint recalc if flagged
    // 2. Damage cooldown decrement
    // 3. Fire damage check (deferred)
    // 4. Wobble animation
    // 5. State dispatch:
    match building.state {
        BuildingState::Active => {
            // Dispatch by type flags: spawn/convert/vehicle
            if building.behavior_flags & 0x20 != 0 { tick_spawn(building); }
            if building.behavior_flags & 0x01 != 0 { tick_convert(building); }
            // Wood consumption + population growth always
            tick_wood_consumption(building);
            tick_pop_growth(building);
        }
        BuildingState::Init => tick_constructing(building),
        BuildingState::Destroying => tick_destroying(building),
        BuildingState::Sinking => tick_sinking(building),
        BuildingState::FinalTeardown => { /* remove from pool */ }
        _ => {}
    }
}
```

### Anti-Patterns to Avoid
- **Putting building logic in UnitCoordinator:** Buildings are a separate subsystem. Don't bloat coordinator; create a BuildingCoordinator or tick buildings through the ObjectTick trait.
- **Duplicating ObjectPool data:** Buildings live in the pool. Don't create a parallel Vec<Building>. Use pool.buildings_mut() iterator (to be added, mirroring persons_mut()).
- **Hardcoding constants:** Use named constants matching original binary names (WOOD_HUT_1, MANA_F_BRAVE, etc.) for traceability.
- **Skipping the cascade:** Terrain modification MUST trigger the full cascade (heights->normals->walkability->water->pathfinding->mesh). Missing any step causes visual or gameplay bugs.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Building footprint checks | Custom footprint collision | Existing `ShapeFootprints` + `is_cell_occupied()` | Already parsed from SHAPES.DAT with 64 entries |
| Terrain flattening | Custom height averaging | Existing `flatten_building_footprint()` | Already matches original 0x42F2A0 |
| Terrain smoothing | Custom interpolation | Existing `smooth_terrain_area()` | Already handles edge transitions |
| Melee damage formula | Custom damage calc | Existing `calculate_melee_damage()` | Already implemented with bloodlust |
| Damage application | Custom HP reduction | Existing `apply_damage()` | Already handles shield halving |
| Pathfinding | Custom A* for buildings | Existing 4-tier pathfinding system | RegionMap already has has_building() flag |
| RNG | std::rand | Existing `GameRng` | Deterministic LCG matching original binary |
| Cell spatial tracking | Custom spatial hash | Existing `CellGrid` | Doubly-linked list per cell, O(1) insert/remove |

**Key insight:** Phase 1 built substantial infrastructure. The risk is re-implementing things that already exist rather than finding and using them.

## Common Pitfalls

### Pitfall 1: Borrow Checker with Pool + Grid + Building Logic
**What goes wrong:** Building tick needs mutable access to its own BuildingData AND read access to other objects (e.g., occupants, nearby enemies). The pool owns everything, creating aliasing issues.
**Why it happens:** ObjectPool stores everything in one Vec<PoolSlot>. You can't get `&mut pool[building]` and `&pool[occupant]` simultaneously.
**How to avoid:** Use index-based patterns. Collect needed data (handles, positions) into local variables first, then mutate. Or split into read-phase and write-phase per tick, like the existing coordinator does for combat.
**Warning signs:** Compile errors about "cannot borrow `pool` as mutable because it is also borrowed as immutable."

### Pitfall 2: Forgetting Terrain Cascade Steps
**What goes wrong:** Height changes don't update walkability, pathfinding, or mesh -- leading to units walking through water or terrain rendering incorrectly.
**Why it happens:** The cascade has 6+ steps and it's tempting to skip some during development.
**How to avoid:** Implement cascade as a single function that always runs all steps. Test with assertion that all derived data is consistent after modification.
**Warning signs:** Visual glitches at modified terrain, units walking on water, buildings floating.

### Pitfall 3: State Machine Transition Ordering
**What goes wrong:** Person enters building but building doesn't register occupant, or occupant count drifts.
**Why it happens:** Building occupant system and person state machine must stay synchronized. If person dies while in building, occupant count must decrement.
**How to avoid:** Always update both sides atomically. Building_EjectPerson must update both the building's occupant_slots AND the person's state. Death handling must check if person is inside a building.
**Warning signs:** Occupant count != actual persons with Housed state. Buildings that think they're full but aren't.

### Pitfall 4: app.rs Size (3296 lines)
**What goes wrong:** Adding building ghost preview, health bars, and new render logic to an already 3296-line file makes it unmaintainable.
**Why it happens:** All render logic lives in app.rs currently.
**How to avoid:** Add building rendering incrementally through the existing FrameState pattern. Keep new rendering code in separate modules (e.g., render/building_preview.rs) and call from app.rs.
**Warning signs:** Single functions exceeding 200 lines, merge conflicts, long compile times.

### Pitfall 5: Not Testing with Original Constants
**What goes wrong:** Economy feels wrong -- units train too fast, mana generates too slowly, population grows too quickly.
**Why it happens:** Using approximate constants instead of exact values from the binary.
**How to avoid:** Extract ALL constants from things-to-implement.md and docs/specs/buildings.md. Name them matching original binary names. Write tests that verify formula outputs match expected values.
**Warning signs:** Game feel diverges from original. Specific numbers in test assertions.

### Pitfall 6: Building vs Person Pool Conflicts
**What goes wrong:** Buildings and persons share the same ObjectPool (1101 max). Heavy building placement could exhaust pool capacity.
**Why it happens:** Original binary had separate high-priority (units/buildings) and low-priority (effects) pools. Our pool is unified.
**How to avoid:** Monitor active_count. In Phase 2, effects aren't implemented yet, so pool capacity is ample. Long-term, may need priority tiers.
**Warning signs:** pool.create() returning None during gameplay.

## Code Examples

### Adding BuildingData to ObjectPool
```rust
// In src/engine/objects/types.rs
// Replace: GameObjectData::Building(())
// With:    GameObjectData::Building(BuildingData)

pub struct BuildingData {
    pub state: BuildingState,
    pub building_subtype: u8,
    pub wood_stored: u16,
    pub occupant_slots: [Option<ObjectHandle>; 6],
    pub occupant_count: u8,
    pub construction_progress: u16,
    pub conversion_countdown: u16,
    pub damage_accumulated: u16,
    pub damage_cooldown: u8,
    pub behavior_flags: u32,
    pub shake_x: i16,
    pub shake_z: i16,
}

// In pool.rs, add buildings() iterator:
pub fn buildings(&self) -> impl Iterator<Item = (ObjectHandle, &ObjectHeader, &BuildingData)> {
    self.slots.iter().enumerate().filter_map(|(i, slot)| {
        if let PoolSlot::Occupied(obj) = slot {
            if let GameObjectData::Building(ref bd) = obj.data {
                return Some((i as ObjectHandle, &obj.header, bd));
            }
        }
        None
    })
}
```

### Building Placement Command
```rust
// In src/engine/command.rs, add:
GameCommand::PlaceBuilding { building_type: u8, cell_x: i32, cell_y: i32, rotation: u16 },
GameCommand::CancelPlacement,
GameCommand::EnterBuildMode { building_type: u8 },
```

### Terrain Modification Cascade
```rust
// In src/engine/terrain/cascade.rs
pub fn terrain_cascade(
    landscape: &mut LandscapeMesh<128>,
    region_map: &mut RegionMap,
    cell_x: usize, cell_y: usize,
    radius: usize,
) {
    // 1. Heights already modified by caller
    // 2. Recalculate normals for affected cells
    landscape.recalculate_normals(cell_x, cell_y, radius);
    // 3. Update walkability (water cells become unwalkable)
    update_walkability(landscape, region_map, cell_x, cell_y, radius);
    // 4. Check buildings on affected cells
    check_buildings_on_modified_terrain(cell_x, cell_y, radius);
    // 5. Update water surface
    update_water_surface(landscape, cell_x, cell_y, radius);
    // 6. Invalidate pathfinding segments
    invalidate_path_segments(cell_x, cell_y, radius);
    // 7. Mark GPU heights buffer dirty
    landscape.mark_heights_dirty(cell_x, cell_y, radius);
}
```

### Person State Extension Pattern
```rust
// In person_state.rs, extend enter_state match:
match new_state {
    // ... existing states ...
    PersonState::EnterBuilding => enter_enter_building(unit),
    PersonState::Housing => enter_housing(unit),
    PersonState::Training => enter_training(unit),
    PersonState::Gathering => enter_gathering(unit),
    PersonState::GatheringWood => enter_gathering_wood(unit),
    PersonState::CarryingWood => enter_carrying_wood(unit),
    _ => { /* Unimplemented states -- no-op */ }
}

// In tick_state match:
match unit.state {
    // ... existing states ...
    PersonState::EnterBuilding => tick_enter_building(unit),
    PersonState::Housing => tick_housing(unit),
    PersonState::Training => tick_training(unit),
    PersonState::Gathering => tick_gathering(unit),
    PersonState::GatheringWood => tick_gathering_wood(unit),
    PersonState::CarryingWood => tick_carrying_wood(unit),
    _ => TickResult::Continue,
}
```

### Projectile/Shot System
```rust
// New Shot type data for ObjectPool
pub struct ShotData {
    pub shot_type: u8,        // Standard=1, Trail=2, Fireball=4, etc.
    pub target_handle: Option<ObjectHandle>,
    pub damage: u16,
    pub aoe_radius: u16,
    pub knockback_force: u16,
    pub lifetime: u16,        // ticks remaining
}

// In pool.rs, GameObjectData::Shot(ShotData)
// Shot_Update: move toward target each tick
// Shot_ProcessImpact: on arrival, apply AOE damage + knockback
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Vec<Unit> direct ownership | ObjectPool with handles | Phase 1 (01-01) | Buildings can share the same pool |
| Separate spatial tracking | CellGrid with doubly-linked lists | Phase 1 (01-02) | Buildings tracked in same grid as units |
| Vec<Unit> compatibility shim | units() accessor on UnitCoordinator | Phase 1 (01-03) | Rendering still works via shim; new building code should use pool directly |

**Note on compatibility shim:** Phase 1 kept Vec<Unit> as a compatibility shim for rendering. Phase 2 building code should work through the ObjectPool directly, not create another shim. Rendering for buildings already goes through Object3D meshes (build_building_meshes in render/buildings.rs).

## Open Questions

1. **Pool Access Sharing Between Building and Unit Coordinators**
   - What we know: UnitCoordinator owns ObjectPool and CellGrid. Buildings need access too.
   - What's unclear: Should BuildingCoordinator be a separate struct, or should we create a shared GameWorld that owns the pool and passes &mut references to both subsystems?
   - Recommendation: Move ObjectPool and CellGrid ownership up to GameWorld (or a new WorldState struct). Pass &mut pool to building_tick() and unit_tick() in sequence within the game loop. This avoids aliasing and matches the original's sequential tick order.

2. **Building Rendering Integration**
   - What we know: build_building_meshes() already renders static building meshes. Ghost preview needs transparency. Construction needs progress animation.
   - What's unclear: How construction progress animation works visually (partial mesh? scaling? transparency fade-in?).
   - Recommendation: Start with simple approach (full mesh + construction scaffold/transparency). Check if original binary uses height scaling (building rises from ground). Can refine in later waves.

3. **Wood Constant Values**
   - What we know: things-to-implement.md lists constant NAMES (WOOD_HUT_1, etc.) but not all VALUES.
   - What's unclear: Exact numeric values for wood costs and mana rates.
   - Recommendation: These values come from the game's constant.dat file (encrypted). Check if the project has extracted values already, or hardcode sensible defaults and validate against gameplay feel. The docs/specs files may have exact values embedded in function analyses.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `cargo test` |
| Config file | Cargo.toml (standard) |
| Quick run command | `cargo test` |
| Full suite command | `cargo test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| BLDG-01 | Building state transitions (init->construction->active->destroying->sinking->teardown) | unit | `cargo test buildings::state_machine` | Wave 0 |
| BLDG-02 | Construction progress + wood consumption | unit | `cargo test buildings::tick::construction` | Wave 0 |
| BLDG-03 | Placement validation (footprint, clearance) | unit | `cargo test buildings::placement` | Wave 0 |
| BLDG-04 | Occupant enter/exit, capacity checks | unit | `cargo test buildings::occupants` | Wave 0 |
| BLDG-05 | Hut brave spawning at correct rates | unit | `cargo test buildings::spawning` | Wave 0 |
| BLDG-06 | Training conversion with costs | unit | `cargo test buildings::training` | Wave 0 |
| BLDG-07 | Building damage and destruction | unit | `cargo test buildings::damage` | Wave 0 |
| BLDG-08 | Building combat (fighter slots, attacks) | unit | `cargo test buildings::combat` | Wave 0 |
| ECON-01 | Wood gathering state cycle | unit | `cargo test economy::wood` | Wave 0 |
| ECON-02 | Wood storage and consumption tracking | unit | `cargo test economy::wood::storage` | Wave 0 |
| ECON-03 | Mana generation rates match original | unit | `cargo test economy::mana` | Wave 0 |
| ECON-04 | Mana pool with MAX_MANA cap | unit | `cargo test economy::mana::pool` | Wave 0 |
| ECON-05 | Population cap from housing | unit | `cargo test economy::population` | Wave 0 |
| PRSN-01 | EnterBuilding state transition | unit | `cargo test person_state::enter_building` | Wave 0 |
| PRSN-02 | ExitBuilding state + facing | unit | `cargo test person_state::exit_building` | Wave 0 |
| PRSN-03 | Housed state contributes to population | unit | `cargo test person_state::housed` | Wave 0 |
| PRSN-04 | Training state with conversion timer | unit | `cargo test person_state::training` | Wave 0 |
| PRSN-05 | Gather wood state cycle | unit | `cargo test person_state::gather_wood` | Wave 0 |
| PRSN-06 | Drown state -> death | unit | `cargo test person_state::tick_drowning` | Exists (pass) |
| PRSN-07 | Guard state holds position | unit | `cargo test person_state::guard` | Wave 0 |
| PRSN-08 | Death cleanup (pool + grid removal) | unit | `cargo test person_state::death_cleanup` | Wave 0 |
| CMBT-01 | Melee damage formula correctness | unit | `cargo test person_state::calculate_melee_damage` | Exists (pass) |
| CMBT-02 | Projectile tracking + AOE | unit | `cargo test combat::projectile` | Wave 0 |
| CMBT-03 | Drum tower auto-attack | unit | `cargo test combat::drum_tower` | Wave 0 |
| CMBT-04 | Death + kill tracking cleanup | unit | `cargo test combat::death_cleanup` | Wave 0 |
| CMBT-05 | Knockback angle-based velocity | unit | `cargo test combat::knockback` | Wave 0 |
| TERR-01 | Height modification gradual change | unit | `cargo test terrain::modify_height` | Wave 0 |
| TERR-02 | Cascade updates all derived data | unit | `cargo test terrain::cascade` | Wave 0 |
| TERR-03 | Water/land toggle based on height | unit | `cargo test terrain::water_interaction` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test`
- **Per wave merge:** `cargo test` (all 289+ existing tests + new tests)
- **Phase gate:** Full suite green before verify-work

### Wave 0 Gaps
- [ ] `src/engine/buildings/` module -- entire new module tree
- [ ] `src/engine/economy/` module -- entire new module tree
- [ ] `src/engine/combat/` module -- projectile/knockback (melee exists)
- [ ] `src/engine/terrain/` module -- modification and cascade (rendering terrain exists in render/)
- [ ] Person state test extensions for new states (EnterBuilding, Housed, Training, GatherWood, Guard)

## Sources

### Primary (HIGH confidence)
- `docs/specs/buildings.md` -- Complete Ghidra disassembly of building system (81KB, byte-level field offsets, state machines, flag definitions)
- `things-to-implement.md` -- Comprehensive inventory of all systems with addresses, constants, status tracking
- `src/engine/objects/pool.rs` -- ObjectPool implementation (Phase 1 output)
- `src/engine/objects/types.rs` -- GameObjectData enum with Building(()) stub to replace
- `src/engine/units/person_state.rs` -- All 44 PersonState values defined, 8 implemented
- `src/engine/units/coordinator.rs` -- UnitCoordinator with pool/grid integration
- `src/engine/state/traits.rs` -- Tick subsystem traits (TerrainTick, ObjectTick, ManaTick, PopulationTick)
- `src/render/terrain.rs` -- flatten_building_footprint() and smooth_terrain_area()

### Secondary (MEDIUM confidence)
- `src/engine/movement/region.rs` -- RegionMap with has_building() flag, walkability checks
- `src/data/objects.rs` -- ShapeFootprints, Shape struct for footprint data
- `src/engine/command.rs` -- GameCommand enum (needs building placement additions)
- `src/engine/frame.rs` -- FrameState (needs building state info for rendering)

### Tertiary (LOW confidence)
- Exact numeric values for wood costs, mana rates, spawn timers -- names documented but some values need extraction from constant.dat or further RE analysis

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- no new dependencies, pure game logic on existing infrastructure
- Architecture: HIGH -- existing patterns (ObjectPool, tick traits, PersonState) provide clear extension points; docs/specs/buildings.md provides byte-level accuracy
- Pitfalls: HIGH -- borrow checker patterns well-understood from Phase 1; cascade ordering from original binary documentation
- Economy constants: MEDIUM -- constant names documented but some exact values may need validation

**Research date:** 2026-03-17
**Valid until:** 2026-04-17 (stable domain -- game RE data doesn't change)
