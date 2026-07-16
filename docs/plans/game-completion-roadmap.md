# Pop3 Game Completion Audit and Roadmap

**Audit date:** 2026-07-12
**Repository state reviewed:** `dba330c` on `main`
**Goal:** turn the current renderer/simulation prototype into a playable and eventually faithful remake of *Populous: The Beginning*.

## Executive summary

The project has a strong technical foundation, but it is not yet a playable game.

What exists today is best described as a level renderer plus a partially connected simulation sandbox:

- Original terrain, palettes, textures, skies, models, sprites, animations, and the fixed 2,000-object level table can be read.
- Campaign levels 1-25 can be selected directly and rendered when the original game files are supplied.
- Person units can be selected, moved, animated, pathfound, and put through basic automatic combat.
- Building, economy, terrain-modification, projectile, effect, HUD, and victory modules contain substantial unit-tested logic.
- The repository builds successfully and all 558 tests pass.

The missing piece is a coherent runtime game world. Most recently implemented gameplay systems are isolated modules or test harnesses rather than features a player can use. Loaded buildings and scenery remain static render objects, player building commands are no-ops, spells cannot be cast, AI scripts are not loaded, campaign objectives are not parsed, and save/load, discovery, menus, vehicles, audio, and campaign progression do not exist.

The immediate priority should therefore be **integration and one complete vertical slice**, not adding another large collection of disconnected subsystem functions.

## Direct answers

### Can we read the planet data from the original files?

**Yes, for the visual world and basic level object placement. Not yet for the full gameplay definition.**

The apparent planet is not stored as a true sphere. It is a 128x128 toroidal heightmap rendered with curvature to create the planet illusion. The current code can read and compose:

- `LEVELS/LEVL2NNN.DAT`: heightmap, three map layers, four tribe records, sunlight, and 2,000 fixed object slots.
- `LEVELS/LEVL2NNN.HDR`: landscape type and object-model bank selection.
- Terrain palettes, displacement maps, textures, cliffs, sky, water, and model banks.
- Unit/building/scenery placement records.
- Person sprites and animation chains.
- A composed globe texture through the `pop_res globe` tool.

However, the loader currently skips or treats as opaque several gameplay-relevant fields:

- The three DAT map layers are skipped rather than modeled as named gameplay data.
- Tribe configuration remains an opaque 16-byte record and is not applied to `GameWorld`.
- Most HDR metadata, markers, availability flags, and level-specific settings are ignored.
- The trailing DAT bytes are not understood.
- `LEVLSPC2.DAT`, `OBJECTIV.DAT`, CPATR patrol data, and CPSCR AI scripts have no parser.
- `constant.dat`, which holds spell, economy, building, and difficulty balance values, has no parser.

So the project can reconstruct the planet visually, but cannot yet reconstruct the complete rules and scripted behavior of a level.

### Can we play the levels?

**They can be loaded, viewed, and partially simulated; they cannot be played to completion.**

With owned original data, the app supports direct launch using `--base` and `--level`. The player can inspect the world, select person units, issue move orders, and observe basic automatic person combat. Level navigation is hard-coded to campaign levels 1-25.

It is not currently possible to play a normal campaign level because:

- There is no functioning building placement interaction.
- Loaded buildings do not enter the dynamic building simulation.
- Wood gathering, housing, training, and population growth are not reachable as a complete player loop.
- Spells cannot be selected, targeted, or cast.
- Enemy AI does not run.
- Objectives and discoveries are not loaded.
- Tribe population and availability state are not initialized from the level.
- Victory/defeat results do not transition to campaign progression.
- There is no campaign menu, level unlock state, or save/load.

The current app is therefore a useful renderer and movement/combat sandbox, not a game session.

### What is the state of spells?

**The spell system itself has not been implemented.**

What exists:

- Spell names and icons in the HUD.
- Placeholder mana costs and an affordability/charge calculation.
- An empty cooldown data contract in `HudState`.
- Terrain-modification, projectile, damage, knockback, building-damage, and effect-pool primitives that spells can eventually call.
- A tested helper that maps 12 spell impact types to effect records.

What does not exist:

- An `engine/spells` module or authoritative spell state.
- A cast command, selected-spell state, targeting mode, range validation, mana deduction, cooldown update, or unlock check.
- Spell objects in the world; the `GameObjectData::Spell` variant is empty.
- Implementations of the 21 original spells.
- Runtime terrain updates and GPU rebuilds caused by spells.
- Rendered spell effects; the effect pool is updated but never consumed by a renderer.
- Values loaded from `constant.dat`; current mana costs are explicitly placeholders.

### What is the state of buildings?

**There is substantial building logic, but almost none of it is usable in the running app.**

Implemented and unit-tested modules cover:

- Building subtypes and state transitions.
- Construction progress and wood consumption.
- Six occupant slots.
- Hut spawning timers.
- Training conversion timers and costs.
- Damage, destruction states, wobble, and building combat actions.
- Placement validation.
- Building mesh rendering, terrain flattening, footprints, and ghost-render pipeline setup.

The runtime integration is incomplete:

- `UnitCoordinator::load_level` imports only persons into the object pool.
- Loaded buildings remain in `level_objects` as static render records.
- `PlaceBuilding`, `EnterBuildMode`, `EnterBuilding`, and `TrainUnit` are accepted by `GameCommand` but handled as no-ops in `GameEngine::apply_command`.
- `FrameState` always reports `ghost_preview: None`.
- HUD building entries are labels only; panel clicks do not issue building commands.
- Hut spawning creates a person in the object pool but not in the live `Vec<Unit>` compatibility representation, so the new brave is not rendered or simulated.
- Training conversion updates the pool representation but not the live unit representation.
- Occupant deferred actions currently pass the building handle as the occupant handle, rather than the person handle.

The building subsystem should be treated as **logic ready for integration**, not as a completed player feature.

## Current capability matrix

| Area | Data/parsing | Engine logic | Runtime integration | Player-facing result |
|---|---|---|---|---|
| Terrain/planet | Strong | Static terrain strong; mutation primitives exist | Initial load works; mutation tick is a no-op | Viewable, not spell-modifiable |
| Textures/sky/water | Strong | Animation/render parameters exist | Rendered | Working visual foundation |
| Persons/animation | Strong | Movement, selection, states, basic combat | Connected through `UnitCoordinator` | Partially interactive |
| Pathfinding | Strong, fixture-backed | Four-tier pathfinding implemented | Connected to move orders | Working for current movement cases |
| Object pool/cell grid | N/A | Implemented | Persons duplicated between pool and `Vec<Unit>` | Not yet authoritative |
| Buildings | Placement records and models read | Substantial tested logic | Loaded buildings are static; commands are no-ops | Not playable |
| Economy | No `constant.dat` parser | Mana, wood, population helpers exist | Mana only partly bridged; population/wood loops incomplete | Not playable |
| Combat | Level persons read | Melee, shots, damage, knockback exist | Person auto-combat partly connected | Sandbox-level only |
| Effects | Some effect types modeled | 512-slot effect pool exists | Updated but not rendered | Invisible |
| HUD | Panel sprite data read | Layout/minimap/info helpers exist | Rendered and partially clickable | Informational, not command-complete |
| Spells | No balance/config parser | No spell manager or spell states | Not connected | Cannot cast |
| AI | No CPATR/CPSCR parser | No AI engine | `AiTick` is `NoOp` | No opponent behavior |
| Objectives/discovery | No parser | No gameplay system | Not connected | No level goals/unlocks |
| Victory/defeat | N/A | Elimination checker exists | Tribe state is not initialized or synchronized | Not meaningful in a session |
| Campaign/menu | No campaign table loader | No frontend/campaign state | Direct CLI level selection only | No campaign experience |
| Save/load | No parser/serializer | None | None | Missing |
| Vehicles/creatures | Placement records can be seen | Empty object variants | Static markers/models at most | Missing gameplay |
| Audio | No SDT/SF2 parser | None | None | Silent |

## What is preventing completion

### 1. Two competing sources of truth for persons

`UnitCoordinator` keeps both an `ObjectPool` representation and a separate `Vec<Unit>` compatibility representation. Movement, selection, animation, combat, and rendering mostly use the vector; mana and building interactions use the pool. The method intended to synchronize from the pool is unused.

This already creates observable breakage for spawned and trained units and will become unmanageable once spells, AI, vehicles, and saves mutate objects. One authoritative simulation representation is required before adding more object types.

### 2. Loaded level objects are not converted into live game objects

Only person records are imported into the dynamic simulation. Buildings, scenery, vehicles, discoveries, effects, general objects, and other types remain static render records. As a result, the engine can contain correct building functions while no loaded building ever calls them.

### 3. The main tick loop is mostly placeholders

In the running app, terrain, water, network/actions, game time, single-player logic, tutorial, AI, and population are all supplied as `NoOp`. Object ticking is connected and mana is manually run afterward as a borrow-checking workaround.

This means the documented faithful tick order exists structurally but not behaviorally. It should be replaced with a real `GameSession`/`GameWorld` owner that can tick all systems without ad hoc bridges.

### 4. Player commands stop at the UI boundary

The command enum includes building actions but the engine ignores them. Spell commands do not exist. The HUD can switch tabs, but its building and spell entries are display-only. A deterministic action queue is needed for building placement, unit orders, training, worship, spell casting, and AI commands.

### 5. Level gameplay data is missing

Visual level loading is ahead of gameplay loading. The project still needs parsers and typed models for:

- `constant.dat`
- `LEVLSPC2.DAT`
- `OBJECTIV.DAT`
- CPATR files
- CPSCR files
- Relevant HDR marker, availability, and campaign fields
- Named DAT map-layer semantics

Without these, balance values are placeholders and campaign behavior must be hard-coded.

### 6. AI is the largest feature gap

Campaign opponents are controlled by original script data plus decision-making code. No AI script loader, interpreter, command scheduler, personality state, target selection, building priorities, or spell policy exists.

The current plan proposes Lua, but the repository contains neither Lua scripts nor a verified conversion pipeline. For a remake that reads original game files, the safer architecture is:

1. Parse CPSCR into a documented internal instruction representation.
2. Implement deterministic value evaluation and the campaign-used opcodes.
3. Validate commands and decisions against original traces.
4. Optionally expose that internal representation to Lua later for modding or maintainability.

Lua should not become a requirement for replacing data already present in the owned original files.

### 7. Campaign completeness conflicts with the existing scope

The planning documents promise a playable 25-level campaign while deferring vehicles, specialist abilities, creatures, remaining spells, and scenery interaction. The same research notes that boats and airships are required by later levels. A complete campaign and a limited “core 12 spells/no vehicles” release cannot both be the same milestone.

The roadmap should explicitly distinguish:

- A playable sandbox.
- One completable campaign level.
- The early campaign.
- All 25 campaign levels.
- Full feature parity with the original game.

### 8. Tests prove components, not a game

The 558 passing tests are valuable, especially for pathfinding and pure engine functions. They do not currently exercise:

- Loading a real level and creating all live objects.
- Issuing a player building command through the app boundary.
- A full gather/build/spawn/train loop.
- Casting a spell and observing a world/render result.
- Running an AI script.
- Winning a level and advancing the campaign.
- Saving and resuming deterministically.

No owned original game data is available at the current default path, so this audit could verify compilation and unit tests but not launch a real level. A developer-owned data fixture or configurable local test-data path is required for end-to-end validation.

### 9. Rendering feedback is incomplete

The effect pool has no render consumer. Building health bars are modeled but not generated. Terrain mutations have no complete dirty-region-to-GPU pipeline. Audio is absent. Even correct simulation code will feel broken without visible and audible feedback.

### 10. Historical tracking has drifted

Superseded historical inventories and phase checklists disagree about completion: some omit recently implemented modules, while others treat isolated unit-tested logic as a playable feature. Progress should be tracked at three levels: **logic**, **runtime integration**, and **playable acceptance test**.

## Recommended completion roadmap

### Milestone 0 — Establish truthful acceptance criteria and fixtures

**Outcome:** repeatable evidence for whether a feature works in a real level.

- Add a data-root configuration used by tests and development tools; do not commit copyrighted assets.
- Add a level-inspection command that reports every parsed section and object count without opening a GPU window.
- Grow `pop_extract` into the named, manifest-backed pipeline for inspecting original structures, construction phases, textures, and animations on demand.
- Add golden summaries for at least levels 1, 5, 10, 15, 20, and 25.
- Add an integration-test harness capable of ticking a loaded level headlessly.
- Define a command replay format for deterministic scenarios.
- Update status tracking to distinguish unit logic from runtime/playable completion.

**Exit test:** a headless test loads an owned Level 1, validates all expected objects/tribes/markers, advances deterministic ticks, and produces a stable state digest.

### Milestone 1 — Make one authoritative runtime world

**Outcome:** every live object is created, updated, queried, and rendered from one source of truth.

- Replace the `Vec<Unit>`/pool split with stable handles and pool-backed person iteration, or use typed arenas behind a single `World` facade.
- Import all level object types into the dynamic world at load time.
- Initialize tribe activity, population, mana, player tribe, spell/building availability, and sunlight from level data.
- Put real subsystem owners into the tick loop instead of `NoOp` placeholders.
- Add a deterministic action queue and route both player and AI commands through it.
- Generate `FrameState` from the authoritative world, including dynamic buildings, effects, shots, and terrain dirty regions.
- Break the 3,700-line `render/app.rs` into app/event handling, session orchestration, and render pipelines.

**Exit test:** loaded persons and buildings tick in place, a newly spawned or converted person appears immediately in simulation and rendering, and no synchronization shim is required.

### Milestone 2 — Deliver a playable economy/combat sandbox

**Outcome:** the player can perform the complete non-spell RTS loop on one map.

- Reconstruct the native left-side building tab and wire its pictographic building cells to build mode, placement validation, ghost preview, resource deduction, and dynamic building creation. Spell and follower tabs remain inert and out of scope for this milestone.
- Wire building-panel clicks to build mode, placement validation, ghost preview, resource deduction, and dynamic building creation.
- Replace the debug/F1 construction controls with the native construction tab. Load the original build-menu glyphs from `hfx0-0.dat` image numbers `1028..1036` in native element order (or output from `pop_extract building-panel-icons`) rather than using generated 3D structure previews; original assets remain extracted from the user's game data and are not committed.
- Render the shoreline as animated water overlapping the land edge with the original-style beach transition instead of a static boundary.
- Wire unit orders for building, entering/exiting, training, gathering, guarding, and attacking.
- Make trees live scenery objects with wood depletion and removal/update behavior.
- Synchronize tribe population, housing capacity, mana, wood, kills, and reincarnation state.
- Complete construction, hut growth/upgrades, brave spawning, training, building damage, occupant ejection, and teardown.
- Preserve animated shore water and blend the adjacent land texture over it with the original-style pixel transition instead of raising zero-height shore cells.
- After construction completion, rendezvous assigned workers outside the entrance in groups capped at six; implement exact slot placement with the later occupancy/exit behavior.
- Render dynamic buildings, projectiles, building/unit health, and gameplay-critical effects.
- Make victory and defeat operate on authoritative tribe/object state.

**Exit test:** from a loaded level, a human can place a hut, gather wood, finish construction, grow population, train a specialist, fight an enemy, destroy a building, and trigger a valid win/loss state.

### Milestone 3 — Implement the spell platform and core spell vertical slice

**Outcome:** spells are a first-class deterministic game system rather than HUD decoration.

- Parse spell costs, ranges, damage, altitude bands, durations, and cooldowns from `constant.dat`.
- Add spell selection, targeting modes, range/terrain/object validation, unlocks, mana deduction, cooldowns, and cast commands.
- Add live spell/effect objects and render their gameplay-critical visuals.
- Connect terrain changes to water, walkability, pathfinding invalidation, buildings, and GPU dirty-region rebuilds.
- First vertical slice: Burn, Blast, Convert Wild, Flatten Land, and Land Bridge.
- Second slice: Lightning, Shield, Teleport, Swamp, Erosion, Earthquake, and Volcano.
- Implement the remaining original spells before claiming feature parity.

**Exit test:** each spell has a deterministic headless scenario plus a visible in-game scenario proving cost, cooldown, target validity, gameplay effect, cleanup, and rendering feedback.

### Milestone 4 — Load and execute original AI data

**Outcome:** campaign tribes build, train, attack, and cast spells from original level scripts.

- Parse CPATR and CPSCR files.
- Document the instruction encoding from representative campaign files.
- Implement script literals, variables, internal attributes, control flow, timers, and subroutines.
- Implement command scheduling, target selection, threats, building priorities, shaman commands, and difficulty modifiers.
- Start with only opcodes reached by Level 1 traces, then expand coverage level by level.
- Make unsupported reached opcodes fail loudly with level/player/instruction context.
- Add deterministic AI trace comparison against the original game where possible.

**Exit test:** Level 1's AI follows a stable build/attack plan, and every reached instruction is implemented and traceable. Repeat for each campaign level before marking it supported.

### Milestone 5 — Make one campaign level complete end to end

**Outcome:** the first honest definition of “playable level.”

- Parse `OBJECTIV.DAT`, `LEVLSPC2.DAT`, and relevant HDR campaign metadata.
- Implement objectives, discoveries/stone-head worship, unlock grants, reincarnation, and post-victory transitions.
- Add the minimum frontend flow: choose data path, start campaign, pause, restart, win/lose, continue.
- Add versioned save/load for the authoritative world and campaign progress. Original save-file compatibility can be a later compatibility target.
- Complete Level 1 from launch to victory and advancement using only normal player input.

**Exit test:** a fresh user can launch the app, start Level 1, receive its objective, play against AI, win or lose, save/load, and advance without CLI level switching.

### Milestone 6 — Expand through all 25 campaign levels

**Outcome:** campaign-complete release.

Progress sequentially and treat each newly reached mechanic as a release gate:

- Add every campaign-used spell and discovery.
- Implement preacher conversion and spy disguise/sabotage when first required.
- Implement boats, airships, production huts, boarding, disembarking, water movement, and flying physics before vehicle-dependent levels.
- Implement required creatures, portals, reincarnation sites, triggers, and scenery interactions.
- Complete level-specific objectives and scripted events.
- Validate balance using values from original data rather than placeholders.
- Maintain a per-level compatibility table with load, AI, objectives, required mechanics, win, save/load, and regression status.

**Exit test:** all 25 levels can be completed in sequence with correct unlocks, objectives, AI behavior, save/load, and ending transition.

### Milestone 7 — Faithful remake completion and release quality

**Outcome:** the project is more than campaign-capable; it is a polished remake.

- Implement all 21 spells and all campaign-visible unit/building/vehicle/scenery behaviors.
- Add SDT sound effects, positional audio, music, ambient audio, and mixer controls.
- Complete effects, animations, fog/visibility, messages, menus, options, and localization-ready text flow.
- Harden file validation and replace data-dependent panics with contextual errors.
- Add Windows, Linux, and macOS CI/build packaging and GPU smoke tests.
- Profile dense battles, effect storms, terrain spells, and long deterministic sessions.
- Defer multiplayer until deterministic single-player state, command replay, and save/load are proven stable.

## Recommended next implementation sequence

The next work should close one vertical slice in this order:

1. Add headless level inspection and owned-data integration fixtures.
2. Create a single authoritative `GameSession`/world owner.
3. Load buildings and scenery into that world, not a static render-only list.
4. Remove the live-person duplication or make one representation strictly derived and read-only.
5. Initialize tribes and synchronize population/economy/victory state.
6. Implement the action queue and make existing building commands real.
7. Make a placed hut construct, render, and spawn a visible simulated brave.
8. Complete a gather/build/train/fight/win scenario.
9. Parse `constant.dat` and implement the first five-spell vertical slice.
10. Parse Level 1 AI/objective data and make Level 1 completable end to end.

This sequence exercises the existing work instead of discarding it and exposes architectural problems before the spell and AI systems multiply the number of object mutations.

## Definitions of done

### Playable sandbox

- A level loads from original data.
- Player units can move, gather, build, train, and fight.
- Population and resources change correctly.
- At least a small core of spells works.
- No AI or campaign progression is required.

### Playable level

- The level's objectives, unlocks, AI, victory, defeat, restart, and save/load work.
- It can be completed through normal player input without debug keys or CLI level switching.

### Campaign complete

- All 25 levels meet the playable-level definition in sequence.
- Campaign unlocks and ending progression work.
- Every mechanic required by those levels is implemented, including vehicles and specialist abilities where used.

### Faithful remake complete

- All original single-player gameplay systems, spells, unit abilities, buildings, vehicles, creatures, scenery interactions, audio, effects, and frontend flows are represented closely enough to pass documented behavioral comparisons.
- Multiplayer may remain a separate post-1.0 milestone if explicitly scoped that way.

## Final assessment

The project is much closer to a viable remake than historical inventories suggested, but farther from playability than historical phase checklists implied. The hard reverse-engineering, rendering, pathfinding, and many subsystem primitives are valuable. The shortest route to a game is now to connect them around one authoritative world, prove one real level end to end, and then expand compatibility level by level.
