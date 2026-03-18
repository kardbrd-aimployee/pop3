# Architecture Research

**Domain:** Classic RTS/God-game engine (Populous: The Beginning reimplementation)
**Researched:** 2026-03-17
**Confidence:** HIGH

## System Overview

The existing 3-layer architecture (data/engine/render) with `GameCommand` input and `FrameState` output boundaries is well-suited for the new gameplay systems. The key architectural question is: where do buildings, spells, combat, AI, effects, and audio fit within this structure, and what are their internal dependencies?

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         Render Layer                                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │ Terrain  │  │ Sprites  │  │ Buildings│  │ Effects  │  │   HUD    │  │
│  │ Renderer │  │ Renderer │  │ Renderer │  │ Renderer │  │ (minimap │  │
│  │          │  │          │  │ (exists) │  │ (NEW)    │  │  spells) │  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘  │
│       │             │             │             │             │          │
│  ┌────┴─────────────┴─────────────┴─────────────┴─────────────┴──────┐  │
│  │                    FrameState (output boundary)                    │  │
│  └───────────────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────────────┤
│                         Engine Layer                                     │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │              GameWorld (tick loop orchestrator)                   │    │
│  │  network -> actions -> game_time -> terrain -> objects ->        │    │
│  │  water -> [single_player/AI -> population -> mana] x N          │    │
│  └──────┬──────────┬──────────┬──────────┬──────────┬──────────┘    │
│         │          │          │          │          │                │
│  ┌──────┴───┐ ┌────┴─────┐ ┌─┴────────┐ ┌┴────────┐ ┌┴────────┐   │
│  │ Object   │ │ Building │ │  Spell   │ │ Combat  │ │   AI    │   │
│  │ Pool     │ │ System   │ │  System  │ │ System  │ │ Script  │   │
│  │ (NEW)    │ │ (NEW)    │ │  (NEW)   │ │ (NEW)   │ │ (NEW)   │   │
│  └──────┬───┘ └────┬─────┘ └──┬───────┘ └┬────────┘ └┬────────┘   │
│         │          │          │          │          │                │
│  ┌──────┴──────────┴──────────┴──────────┴──────────┴───────────┐   │
│  │           UnitCoordinator (exists) + Effect/Audio Managers    │   │
│  └──────────────────────────────────────────────────────────────┘   │
│  ┌────────────┐  ┌────────────┐                                     │
│  │   Audio    │  │  Effect    │                                     │
│  │  Manager   │  │  Manager   │                                     │
│  │  (NEW)     │  │  (NEW)     │                                     │
│  └────────────┘  └────────────┘                                     │
├─────────────────────────────────────────────────────────────────────────┤
│                         Data Layer                                       │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │ Level    │  │ Objects  │  │Animation │  │  Audio   │  │  Script  │  │
│  │ (exists) │  │ (exists) │  │ (exists) │  │  Data    │  │  Data    │  │
│  │          │  │          │  │          │  │  (NEW)   │  │  (NEW)   │  │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘  └──────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Layer | Status |
|-----------|----------------|-------|--------|
| Object Pool | Unified pool of all 11 model types with cell-grid spatial indexing | Engine | NEW -- foundational |
| Building System | Building state machine, construction, occupants, population spawning, damage | Engine | NEW |
| Spell System | 21 spells, mana economy, cooldowns, targeting, spell effect dispatch | Engine | NEW |
| Combat System | Melee damage, projectile (shot) objects, knockback, building combat | Engine | NEW |
| AI Script Engine | Bytecode interpreter, 200+ opcodes, per-tribe personality, decision loop | Engine | NEW |
| Effect Manager | 93 effect types, particle lifetime, terrain modification effects | Engine | NEW |
| Audio Manager | 3D positional audio, SFX dispatch, distance attenuation, music | Engine/Render boundary | NEW |
| UnitCoordinator | Owns live person units, movement, selection, animation | Engine | EXISTS |
| GameWorld | Tick loop orchestrator, subsystem dispatch in original binary order | Engine | EXISTS |
| FrameState | Read-only snapshot for renderer -- must expand to include new systems | Engine | EXISTS (expand) |
| GameCommand | Input boundary -- must expand for spell casting, building placement | Engine | EXISTS (expand) |

## Recommended Project Structure

```
src/
├── data/                       # Binary format parsing (exists)
│   ├── landscape/              # Terrain data (exists)
│   ├── audio.rs                # NEW: SDT/SoundFont parsing
│   ├── scripts.rs              # NEW: AI script bytecode loading
│   ├── constants.rs            # NEW: constant.dat parser (spell costs, damage tables)
│   └── ...                     # existing files unchanged
├── engine/                     # Game simulation (exists)
│   ├── state/                  # GameWorld, tick loop, tribes (exists)
│   ├── movement/               # 4-tier pathfinding (exists)
│   ├── units/                  # UnitCoordinator, person state (exists)
│   ├── objects/                # NEW: unified object pool + cell grid
│   │   ├── mod.rs              # ObjectPool, ObjectId, cell spatial grid
│   │   ├── object.rs           # Base GameObject struct (11 model types)
│   │   └── pool.rs             # Allocation, free lists, type dispatch
│   ├── buildings/              # NEW: building system
│   │   ├── mod.rs              # BuildingManager, building tick
│   │   ├── state_machine.rs    # Construction/Operating/Damaged/Fire/Sinking
│   │   ├── occupants.rs        # Enter/exit, training, population spawning
│   │   └── combat.rs           # Building_ProcessFightingPersons
│   ├── spells/                 # NEW: spell system
│   │   ├── mod.rs              # SpellManager, mana economy
│   │   ├── targeting.rs        # Target validation, range checks
│   │   ├── effects.rs          # Per-spell effect processors (Blast, Lightning, etc.)
│   │   └── constants.rs        # Spell tables (costs, ranges, power)
│   ├── combat/                 # NEW: combat system
│   │   ├── mod.rs              # Damage dispatch, fight damage table
│   │   ├── melee.rs            # Combat_ProcessMeleeDamage formula
│   │   ├── projectiles.rs      # Shot objects, impact processing
│   │   └── knockback.rs        # Knockback physics
│   ├── ai/                     # NEW: AI scripting
│   │   ├── mod.rs              # AI_UpdateAllTribes entry point
│   │   ├── interpreter.rs      # Bytecode interpreter, opcode dispatch
│   │   ├── opcodes.rs          # 200+ opcode implementations
│   │   ├── attributes.rs       # Script value evaluation (types 0/1/2)
│   │   └── personality.rs      # Per-tribe AI personality traits
│   ├── effects/                # NEW: visual/gameplay effects
│   │   ├── mod.rs              # EffectManager, effect tick
│   │   ├── terrain_effects.rs  # Earthquake, Volcano, Land Bridge
│   │   ├── spell_effects.rs    # Spell visual effects (fire, lightning arcs)
│   │   └── particles.rs        # Particle system (smoke, debris)
│   ├── audio/                  # NEW: audio state management
│   │   ├── mod.rs              # AudioManager, Sound_Play equivalent
│   │   └── sound_table.rs      # Sound data table, distance attenuation
│   ├── command.rs              # GameCommand enum (expand)
│   └── frame.rs                # FrameState struct (expand)
└── render/                     # GPU rendering (exists)
    ├── app.rs                  # Main render loop (exists -- needs refactoring)
    ├── effects/                # NEW: effect rendering
    │   ├── mod.rs              # Effect render pipeline
    │   ├── particles.rs        # Particle billboard rendering
    │   └── spell_visuals.rs    # Spell-specific visual effects
    ├── audio/                  # NEW: audio output
    │   ├── mod.rs              # cpal/rodio integration, mixer
    │   └── music.rs            # SoundFont music player
    ├── hud/                    # HUD (exists -- expand)
    │   └── mod.rs              # Add spell bar, mana display, minimap icons
    └── ...                     # existing render files unchanged
```

### Structure Rationale

- **engine/objects/:** The object pool is the foundational system. In the original binary, everything (persons, buildings, creatures, vehicles, scenery, effects, shots, spells) is a "game object" in a unified pool with cell-based spatial indexing. This must exist before buildings, spells, or effects can be created.
- **engine/buildings/ separate from engine/objects/:** Buildings have complex enough state (5 states, occupant management, training queues, population spawning) to warrant their own module, but they are objects in the pool.
- **engine/combat/ separate from engine/units/:** Combat logic (damage formulas, projectiles, knockback) is shared across person melee, building combat, spell damage, and shot impacts. Centralizing it avoids duplication.
- **engine/audio/ in engine, render/audio/ in render:** Audio state (what to play, 3D positions, distance attenuation) is engine concern. Actual PCM output (cpal/rodio) is render-side, matching the existing engine/render boundary pattern.

## Architectural Patterns

### Pattern 1: Trait-Based Tick Subsystems (Existing -- Extend)

**What:** Each new gameplay system implements a tick trait from `engine/state/traits.rs` and plugs into `TickSubsystems`. The tick loop calls them in the original binary's exact order.

**When to use:** Every new system that needs per-tick updates. This is non-negotiable -- the original binary's call order determines game behavior.

**Trade-offs:** Rigid ordering (pro: faithful; con: can't easily parallelize). The trait approach allows NoOp stubs during incremental development.

**Current tick order (must be preserved):**
```
1. network       -> NetworkTick        (stub for now)
2. actions       -> ActionTick         (NEW: building placement, spell cast commands)
3. game_time     -> GameTimeTick       (stub for now)
4. terrain       -> TerrainTick        (NEW: height modification from spells/effects)
5. objects       -> ObjectTick         (EXISTS: UnitCoordinator; EXPAND to full ObjectPool)
6. water         -> WaterTick          (stub for now)
7. [inner loop x (ai_update_mult + 1)]:
   a. single_player -> SinglePlayerTick  (NEW: script-driven level events)
   b. ai            -> AiTick            (NEW: AI_UpdateAllTribes bytecode interpreter)
   c. population    -> PopulationTick    (NEW: housing-based population spawning)
   d. mana          -> ManaTick          (NEW: mana accumulation/spell cooldowns)
```

**Integration approach:** The `ObjectTick` trait currently backed by `UnitCoordinator` must evolve. The new `ObjectPool` should implement `ObjectTick` and internally dispatch to persons (via UnitCoordinator), buildings (via BuildingManager), effects (via EffectManager), etc. This matches the original's `Tick_UpdateObjects` which iterates the object linked list and calls type-specific update functions.

### Pattern 2: Unified Object Pool with Type Dispatch

**What:** A single `ObjectPool` owns all game objects (max 1101 active). Each object has a `model_type` (1-11) that determines its update function, state machine, and rendering. Cell-based spatial grid (128x128) enables O(1) neighbor queries.

**When to use:** This is the core data structure. Every gameplay system creates, queries, or destroys objects through this pool.

**Trade-offs:** Faithful to original (pro: correct behavior); heterogeneous pool means some indirection (con: slightly less cache-friendly than separate typed arrays). But the original only has 1101 objects max, so performance is not a concern.

**Critical integration point:**
```rust
// Object pool dispatches updates by model type
impl ObjectTick for ObjectPool {
    fn tick_update_objects(&mut self) {
        for obj in self.active_objects() {
            match obj.model_type {
                ModelType::Person   => self.person_update(obj),    // delegates to UnitCoordinator
                ModelType::Building => self.building_update(obj),  // delegates to BuildingManager
                ModelType::Effect   => self.effect_update(obj),    // delegates to EffectManager
                ModelType::Shot     => self.shot_update(obj),      // projectile physics
                ModelType::Spell    => self.spell_update(obj),     // spell lifetime/effect
                ModelType::Creature => self.creature_update(obj),
                ModelType::Vehicle  => self.vehicle_update(obj),
                _ => {} // scenery, shapes, etc. are static
            }
        }
    }
}
```

### Pattern 3: GameCommand Expansion for New Input Types

**What:** New gameplay actions (cast spell, place building, train unit) are added as `GameCommand` variants. Mouse clicks on the spell bar, building menu, etc. are resolved in the render layer to specific `GameCommand` values before reaching the engine.

**When to use:** Every new player action.

**Trade-offs:** Keeps input decoupled from engine logic (pro). Command enum grows large (con: manageable with categorized variants).

**New commands needed:**
```rust
enum GameCommand {
    // ... existing commands ...

    // Building
    PlaceBuilding { building_type: u8, cell_x: u8, cell_y: u8 },
    CancelBuilding,

    // Spells
    CastSpell { spell_id: u8, target_x: f32, target_z: f32 },
    SelectSpell(u8),
    ChargeSpell,   // shaman charge-up

    // Unit orders
    OrderAttack { target_unit: UnitId },
    OrderGuard { target_x: f32, target_z: f32 },
    OrderConvert,
    OrderPatrol { waypoints: Vec<(f32, f32)> },
    OrderEnterBuilding { building_id: ObjectId },

    // Training
    TrainUnit { building_id: ObjectId, unit_subtype: u8 },
}
```

### Pattern 4: FrameState Expansion for New Renderable State

**What:** `FrameState` gains references to new system state that the renderer needs: active effects list, audio event queue, building construction progress, spell charge indicators, mana bars.

**When to use:** Whenever a new engine system produces something visible or audible.

**Critical design rule:** FrameState contains ONLY borrowed references and simple copied values. No GPU types. No owned data. This is the existing pattern and must be preserved.

```rust
pub struct FrameState<'a> {
    // ... existing fields ...

    // NEW: Buildings
    pub building_states: &'a [BuildingRenderState],  // position, type, construction %, on_fire

    // NEW: Effects
    pub active_effects: &'a [EffectRenderState],     // position, type, frame, alpha

    // NEW: Spells
    pub spell_charge: Option<SpellChargeState>,       // charging indicator for HUD
    pub mana: &'a [u32; 4],                           // per-tribe mana for HUD

    // NEW: Audio (engine tells render what to play)
    pub audio_events: &'a [AudioEvent],               // (sound_id, position, flags)

    // NEW: Combat
    pub projectiles: &'a [ProjectileRenderState],     // position, type, angle for shot rendering
}
```

## Data Flow

### Gameplay Tick Data Flow

```
GameWorld::run_one_tick()
    |
    +--> ActionTick::tick_process_actions()
    |        Player commands queued last frame are executed:
    |        PlaceBuilding -> ObjectPool::create(Building, ...)
    |        CastSpell -> SpellManager::begin_cast(...)
    |        TrainUnit -> BuildingManager::queue_training(...)
    |
    +--> TerrainTick::tick_update_terrain()
    |        Process pending height modifications from spells/effects
    |        Update walkability grid (affects pathfinding)
    |
    +--> ObjectTick::tick_update_objects()
    |        ObjectPool iterates all active objects by type:
    |        ├── Persons: state machine tick, movement, combat detection
    |        ├── Buildings: construction progress, spawn population, training
    |        ├── Effects: lifetime countdown, terrain modification, particle spawn
    |        ├── Shots: projectile physics, impact detection, damage
    |        └── Spells: active spell effects (expanding shockwave, etc.)
    |
    +--> [Inner loop]:
    |    ├── AiTick: bytecode interpreter runs scripts for each AI tribe
    |    |       Scripts issue commands: build, train, cast spell, attack
    |    |       Commands go through same ActionTick queue as player commands
    |    ├── PopulationTick: housing buildings spawn new braves
    |    └── ManaTick: per-tribe mana accumulation, spell cooldown timers
    |
    +--> Victory check
```

### Audio Event Flow (Engine to Output)

```
Engine systems generate AudioEvents:
    Combat hit    -> AudioEvent { sound_id: 0x42, pos: hit_pos, flags: 0 }
    Spell cast    -> AudioEvent { sound_id: spell_sound[id], pos: caster_pos, flags: 0 }
    Building burn -> AudioEvent { sound_id: 0xAF, pos: building_pos, flags: LOOPING }

FrameState carries &[AudioEvent] to render layer.
Render/Audio module:
    1. Compute distance from camera to each event source
    2. Apply distance attenuation (max range: 0x9000000 squared)
    3. Mix and output via cpal/rodio
```

### Spell Cast Flow (Complete Example)

```
Player clicks spell bar icon
    -> Render layer resolves to GameCommand::SelectSpell(4)  // Whirlwind
    -> Engine stores selected_spell = 4

Player clicks target location on terrain
    -> Render layer ray-casts to world coords
    -> GameCommand::CastSpell { spell_id: 4, target_x, target_z }
    -> Engine::apply_command():
        1. SpellManager::validate_cast(tribe, spell_id, mana) -> Ok/Err
        2. SpellManager::deduct_mana(tribe, spell_cost)
        3. ObjectPool::create(ModelType::Spell, subtype=4, pos=target)
        4. AudioEvent: spell_cast_sound

Next tick, ObjectTick processes the Spell object:
    -> Spell_ProcessShockwave(): expanding damage ring
    -> Each tick: find objects in expanding radius
    -> Apply knockback + damage via Combat::apply_damage()
    -> Create Effect objects (visual particles) via ObjectPool::create(ModelType::Effect, ...)
    -> AudioEvents for impact sounds
    -> After duration expires: ObjectPool::destroy(spell_obj)
```

### Building Construction Flow

```
Player clicks build icon + target cell
    -> GameCommand::PlaceBuilding { type: Hut, cell_x, cell_y }
    -> Engine::apply_command():
        1. Validate: enough braves nearby? Flat terrain? No overlap?
        2. ObjectPool::create(ModelType::Building, subtype=Hut, pos)
        3. Building state = Construction (0x02)
        4. Mark walkability grid cells as unwalkable
        5. Nearby idle braves enter state GoToPoint -> Building

Building tick (in ObjectTick):
    -> Construction state: increment progress counter
    -> When threshold reached: state = Operating (0x03)
    -> Housing type: start population spawning timer
    -> Training type: accept training queue commands
```

## Build Order (Dependency Chain)

This is the critical section for roadmap planning. Systems have hard dependencies on each other.

### Dependency Graph

```
                    ┌──────────────┐
                    │ Object Pool  │ <-- MUST BUILD FIRST
                    │ + Cell Grid  │
                    └──────┬───────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
     ┌────────▼───┐ ┌─────▼──────┐ ┌──▼──────────┐
     │ Building   │ │  Combat    │ │   Effect    │
     │ System     │ │  System    │ │   Manager   │
     │ (states,   │ │ (melee,    │ │ (lifetime,  │
     │  occupants)│ │  damage)   │ │  particles) │
     └────────┬───┘ └─────┬──────┘ └──┬──────────┘
              │           │           │
              │     ┌─────▼──────┐    │
              │     │ Projectile │    │
              │     │ System     │    │
              │     │ (shots)    │    │
              │     └─────┬──────┘    │
              │           │           │
              ├───────────┼───────────┤
              │           │           │
         ┌────▼───────────▼───────────▼────┐
         │         Spell System            │
         │ (uses combat, effects, objects) │
         └────────────────┬────────────────┘
                          │
                   ┌──────▼───────┐
                   │  AI Script   │
                   │  Engine      │
                   │ (commands    │
                   │  all above)  │
                   └──────┬───────┘
                          │
                   ┌──────▼───────┐
                   │    Audio     │
                   │  (triggered  │
                   │   by all     │
                   │   above)     │
                   └──────────────┘
```

### Recommended Build Order

| Order | System | Rationale | Depends On |
|-------|--------|-----------|------------|
| 1 | **Object Pool + Cell Grid** | Everything else creates objects. Without this, no buildings, no effects, no spells. Replaces current ad-hoc unit storage with unified 1101-object pool. | Existing data layer |
| 2 | **Building System** | Buildings are the primary object type after persons. Population spawning (housing) and training (warrior huts, temples) are prerequisites for meaningful gameplay. Also blocks walkability grid updates. | Object Pool |
| 3 | **Combat System** | Melee damage formulas, health tracking, death processing. Already partially implemented in person_state.rs but needs centralization. Required before spells can deal damage. | Object Pool |
| 4 | **Effect Manager** | Visual effects have simple lifecycles (spawn, animate, die) but are created by everything else (combat hits, spell impacts, building fires). Build the container early so other systems can spawn effects. | Object Pool |
| 5 | **Spell System** | Requires combat (for damage), effects (for visuals), buildings (for targeting), and object pool (for spell objects). This is the most integration-heavy system. | Combat, Effects, Buildings, Object Pool |
| 6 | **AI Script Engine** | The AI issues the same commands as the player (build, train, cast, attack). It needs all gameplay systems to exist so its opcodes have something to command. 200+ opcodes makes this the largest single system. | All gameplay systems |
| 7 | **Audio** | Purely additive. Every system above generates audio events. Can be added last without affecting any game logic. No system depends on audio. | None (but triggered by all) |

### Why This Order

1. **Object Pool first** is non-negotiable. The current `UnitCoordinator` manages persons in a `Vec<Unit>`. The original binary uses a unified pool where persons, buildings, effects, and shots coexist with cross-references (e.g., a person's `linked_obj_id` can point to a building or vehicle). The pool must exist before any new object type can be created.

2. **Buildings before Spells** because: (a) buildings provide population growth (housing) which is needed for meaningful gameplay loops, (b) several spells target buildings specifically (Lightning targets buildings first, Earthquake creates buildings), (c) building combat (guard towers) is a combat system feature that exercises the combat code.

3. **Combat before Spells** because every offensive spell ultimately calls `Object_ApplyDamage` or `Combat_ProcessMeleeDamage`. The damage pipeline must exist.

4. **Effects early** because they are simple objects with short lifecycles but are spawned by combat (blood splatter), spells (fire, lightning arcs), buildings (smoke, construction dust), and terrain changes. Having the container ready means other systems can spawn visual feedback immediately.

5. **AI last among gameplay systems** because AI scripts command every other system. An opcode like `DO_BUILD_BUILDING` needs the building system. `DO_CAST_SPELL` needs the spell system. Building AI before its dependencies exist means writing dead code.

6. **Audio truly last** because it is the only system with zero downstream dependents. Every other system generates audio events, but none consume audio output. It is pure polish.

## Anti-Patterns

### Anti-Pattern 1: Separate Object Pools per Type

**What people do:** Keep persons in `Vec<Unit>`, buildings in `Vec<Building>`, effects in `Vec<Effect>` as completely separate collections.

**Why it's wrong:** The original binary uses a single linked list with cross-references by object index. A person's `linked_obj_id` might point to a building (entering it), a vehicle (riding it), or an effect (being affected by it). Separate pools break these cross-references and make the cell-based spatial grid (which indexes ALL object types) impossible to implement faithfully.

**Do this instead:** Unified `ObjectPool` with a shared `ObjectId` space. Type-specific data stored via enum variants or a `TypeData` union within each object. The existing `UnitCoordinator` becomes a subsystem that the `ObjectPool` delegates to for person-type objects.

### Anti-Pattern 2: Implementing AI Before Gameplay Systems

**What people do:** Build the AI interpreter early because it seems like a self-contained system (just parse bytecode and execute opcodes).

**Why it's wrong:** AI opcodes ARE the gameplay systems. Opcode `DO_CAST_SPELL` calls `SpellManager::cast()`. Opcode `DO_BUILD_BUILDING` calls `BuildingManager::place()`. Implementing the interpreter without the systems it commands means either: (a) 200 stub functions that do nothing, or (b) constant refactoring as each system comes online.

**Do this instead:** Build AI last among gameplay systems. Each system provides a clean API that AI opcodes call. The interpreter is straightforward once all APIs exist.

### Anti-Pattern 3: Audio Mixed into Engine Logic

**What people do:** Call `audio.play_sound(0x42)` directly from combat code, spell code, building code.

**Why it's wrong:** Violates the engine/render boundary. Audio output (cpal/rodio) is a render-side concern. Mixing it into engine logic makes the engine untestable without audio hardware and creates circular dependencies.

**Do this instead:** Engine systems push `AudioEvent` structs onto a frame-scoped queue. `FrameState` carries `&[AudioEvent]` to the render layer. The render-side audio module processes the queue. Engine tests never touch audio hardware.

### Anti-Pattern 4: Monolithic app.rs Expansion

**What people do:** Keep adding new rendering code directly into `app.rs` (already 3296 lines / 142KB).

**Why it's wrong:** The file is already the largest in the project. Adding effect rendering, spell visuals, expanded HUD (spell bar, mana), and audio output into it will push it past 5000 lines and make it unnavigable.

**Do this instead:** Extract effect rendering into `render/effects/`, audio output into `render/audio/`, and expanded HUD features into `render/hud/` as separate modules. `app.rs` calls into these modules during the appropriate render pass. This is the highest-priority refactoring task before adding new render features.

## Integration Points

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| Player input -> Engine | `GameCommand` enum variants | Expand enum for spell/building/training commands |
| Engine -> Renderer | `FrameState` borrowed refs | Expand struct for effects, audio events, building states |
| ObjectPool -> UnitCoordinator | Delegation by model_type | Pool owns allocation; coordinator owns person logic |
| ObjectPool -> BuildingManager | Delegation by model_type | Pool owns allocation; manager owns building state machine |
| Combat -> All damage sources | Centralized `apply_damage()` | Melee, spells, shots, building combat all funnel here |
| AI Scripts -> Gameplay systems | Same `GameCommand`/API calls as player | AI is just another input source, not a special path |
| Engine systems -> Audio | `AudioEvent` queue in FrameState | Engine never touches audio hardware |
| Spells -> Terrain | Height modification requests | Queued during spell tick, applied during terrain tick next frame |

### Object Pool <-> Existing UnitCoordinator Migration

The biggest integration challenge is migrating from the current `UnitCoordinator` (owns `Vec<Unit>`) to the unified `ObjectPool`. Strategy:

1. Build `ObjectPool` alongside `UnitCoordinator` initially
2. Move person allocation from `Vec<Unit>` into `ObjectPool` slots
3. `UnitCoordinator` becomes a thin wrapper that borrows person-type objects from the pool
4. Unit IDs become `ObjectId` values (same namespace as buildings, effects)
5. Selection, movement, and state machine code stays in `units/` but operates on pool-borrowed objects

This migration should happen as step 1 (Object Pool) and must be carefully planned to avoid breaking the 260 existing tests.

## Scaling Considerations

Not applicable in the traditional web-service sense, but relevant for game performance:

| Concern | Current (viewer) | Full gameplay | Mitigation |
|---------|-------------------|---------------|------------|
| Object count | ~200 persons | Up to 1101 objects | Fixed pool size, no allocation during gameplay |
| Effect particles | 0 | Up to 640 (low-priority pool) | Low-priority pool with eviction for oldest effects |
| AI script execution | 0 | 3 AI tribes x (ai_update_mult+1) per tick | Budget per-tribe, bail early if over time |
| Audio mixing | 0 | 16-32 simultaneous sounds | Fixed channel count, priority-based eviction |
| Cell grid queries | Unused | Every combat/spell/effect check | O(1) cell lookup, iterate only occupied cells |

The original binary ran on a Pentium II at 10-12 ticks/second with all systems active. Performance will not be a concern on modern hardware.

## Sources

- Original binary reverse engineering specs: `docs/specs/object_system.md`, `docs/specs/buildings.md`, `docs/specs/spells.md`, `docs/specs/combat_and_pathfinding.md`, `docs/specs/ai_scripting.md`, `docs/specs/water_and_effects.md`, `docs/specs/audio.md`
- Existing codebase: `src/engine/state/tick.rs` (tick loop), `src/engine/state/traits.rs` (subsystem traits), `src/engine/units/coordinator.rs` (current unit management), `src/engine/command.rs` (input boundary), `src/engine/frame.rs` (output boundary)
- Original binary address annotations throughout codebase (Ghidra RE of popTB.exe)

---
*Architecture research for: Populous: The Beginning reimplementation -- gameplay systems integration*
*Researched: 2026-03-17*
