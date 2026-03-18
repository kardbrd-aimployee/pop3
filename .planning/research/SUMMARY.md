# Project Research Summary

**Project:** Pop3 (Populous: The Beginning reimplementation)
**Domain:** Classic RTS/God-game engine -- gameplay systems layer
**Researched:** 2026-03-17
**Confidence:** HIGH

## Executive Summary

Pop3 is a Rust+wgpu reimplementation of Populous: The Beginning, currently at ~25% completion (91/366 items). The engine foundation is solid: rendering, pathfinding, unit movement, sprite animation, camera, and level loading all work. What remains is the entire gameplay layer -- the systems that turn a tech demo into a playable 25-level campaign. Research across stack, features, architecture, and pitfalls converges on a clear conclusion: the project needs minimal external dependencies (most systems are custom implementations matching reverse-engineered binary behavior) and must follow a strict build order dictated by hard data dependencies between subsystems.

The recommended approach is bottom-up construction anchored on a unified object pool. Every gameplay system (buildings, spells, combat, effects, AI) creates and queries objects through this pool. The pool must faithfully replicate the original binary's fixed-size array with linked-list free lists and two-tier priority allocation -- this is non-negotiable for determinism. From the pool, systems are built in dependency order: buildings (economy loop), combat (damage pipeline), effects (visual feedback), spells (integration-heavy), then AI scripting (commands everything else). Audio is purely additive and comes last. The HUD/UI track is independently parallelizable.

The dominant risk is determinism divergence. The original game's AI scripts, combat resolution, and game state all depend on objects existing at specific pool indices, the RNG being called in exact order, and subsystems ticking in the original binary's sequence. Three pitfalls are catastrophic if not addressed early: wrong object pool allocation order (cascades into everything), tick subsystem ordering divergence (subtle state corruption), and AI bytecode interpreter off-by-one errors (makes AI brain-dead). All three have HIGH recovery cost. The mitigation strategy is fixture testing against traces from the original binary via Frida instrumentation.

## Key Findings

### Recommended Stack

The stack is remarkably lean. Most gameplay systems are custom implementations -- the original game uses proprietary formats and bespoke logic that no library can replace. Only three areas need external crates.

**Core technologies:**
- **cpal** (0.17.3): Low-level audio output -- needed because the original game's 3D audio math (QSWaveMix distance attenuation, pan calculation) must be replicated exactly, ruling out higher-level libraries like kira/rodio that impose their own spatial models
- **rustysynth** (1.3.6): SF2 SoundFont synthesis -- the only pure-Rust SF2 synth, used for music playback from the original's popfight.sf2
- **serde + bincode** (1.0.228 / 3.0.0): For the new internal save format only -- the original save format is fixed-layout binary handled by bytemuck (already in the project)

**Explicitly NOT using external libraries for:** AI scripting (custom bytecode VM, 200+ opcodes), visual effects (CPU sprite objects, not GPU particles), building state machines (enum + match, 5-10 states). These are all custom because the original game's formats and behaviors are fully reverse-engineered and no generic library would be faithful.

### Expected Features

**Must have (table stakes for playable campaign):**
- Core Object System (unified pool, cell grid) -- foundation for everything
- Building System (construction, occupants, population spawning, wood consumption)
- Person State Extensions (enter/exit building, gather wood, training, drowning)
- Wood Economy + Population System + Mana System (the three interlocking economy loops)
- Combat System (melee, projectiles, knockback, death)
- Spell System (core 12 of 21 spells -- enough for all campaign levels)
- AI/Scripting (bytecode interpreter -- without this, campaign levels are empty sandboxes)
- HUD/UI (minimap, spell bar, mana/population display, health bars)
- Victory/Defeat + Campaign Progression + Discovery System
- Save/Load, Menu System, Font/Text Rendering, Terrain Modification, Minimal Effects

**Should have (v1.x after campaign works):**
- Remaining 9 spells, Vehicle System (boats/airships for late campaign), Audio System, Creature System, Full Effect System, Spy/Preacher abilities

**Defer (v2+):**
- Multiplayer (lockstep networking), Replay System, Custom Key Bindings, Multi-language, Tutorial

### Architecture Approach

The existing 3-layer architecture (data/engine/render) with `GameCommand` input and `FrameState` output boundaries extends naturally. New gameplay systems slot into the engine layer as tick subsystems, called in the original binary's exact order. The key architectural element is the unified `ObjectPool` -- a fixed-size array of 1101 slots with type-dispatch to subsystem managers (BuildingManager, EffectManager, etc.). The existing `UnitCoordinator` becomes a delegate for person-type objects within the pool.

**Major components:**
1. **ObjectPool + Cell Grid** -- unified storage for all 11 object types with O(1) spatial queries
2. **BuildingManager** -- construction state machine, occupant tracking, population spawning, training queues
3. **SpellManager** -- 21 spells, mana economy, cooldowns, targeting, effect dispatch
4. **Combat System** -- centralized damage pipeline shared by melee, projectiles, spells, and building combat
5. **AI Script Engine** -- bytecode interpreter, 200+ opcodes, per-tribe personality, decision loop
6. **EffectManager** -- 93 effect types as sprite objects in a capped pool (512 active + 128 low-priority)
7. **AudioManager** -- async command queue to cpal, 3D positional audio, SF2 music

### Critical Pitfalls

1. **Object pool allocation order breaks determinism** -- Must use fixed-size array with linked-list free lists matching original's two-tier structure. Fixture-test pool indices against original binary traces. Recovery cost: HIGH.
2. **Tick subsystem ordering divergence** -- Subsystems must tick in the original binary's exact order. Each new system inserts at its documented position, never appended arbitrarily. Recovery cost: MEDIUM.
3. **AI bytecode interpreter off-by-one** -- 200+ opcodes with varying operand sizes. A single PC advancement error corrupts all subsequent execution. Implement ALL opcodes (panicking stubs, not silent no-ops). Write a disassembler before the interpreter. Recovery cost: HIGH.
4. **Spell-terrain cascade failures** -- Terrain modification must trigger a complete cascade (heightmap, normals, walkability, building checks, water, pathfinding). Implement as single function, not scattered calls. Recovery cost: MEDIUM.
5. **Building occupant tracking desyncs** -- Use generation counters on pool slots. Eject occupants BEFORE destroying buildings. Mirror original's destroy order. Recovery cost: HIGH.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Core Object System + Cell Grid
**Rationale:** Every gameplay system depends on this. The existing `UnitCoordinator` with `Vec<Unit>` cannot support buildings, effects, or spells. This is the single hardest architectural change because it migrates existing person management into the new pool without breaking 260 existing tests.
**Delivers:** Unified ObjectPool (1101 slots, two-tier free lists), cell-based spatial grid (128x128), ObjectId with generation counters, UnitCoordinator migration to pool delegate.
**Addresses:** Core Object System (FEATURES P1), UnitCoordinator migration (ARCHITECTURE)
**Avoids:** Pitfall 1 (allocation order), Pitfall 7 (pool exhaustion design)

### Phase 2: Building System + Economy
**Rationale:** Buildings unlock the core gameplay loop: population growth, wood economy, training, and mana generation. Without buildings, there is no game -- just units walking around.
**Delivers:** Building state machine (5 states), construction, occupant tracking, population spawning from huts, wood gathering/consumption, building placement UI with ghost preview.
**Addresses:** Building System, Wood Economy, Population System, Building Placement UI (FEATURES P1)
**Avoids:** Pitfall 6 (occupant tracking), Pitfall 4 (person state transitions for enter/exit building)

### Phase 3: Combat System + Projectiles
**Rationale:** Combat is prerequisite for spells (all offensive spells call `apply_damage`), AI (AI issues attack commands), and victory/defeat (elimination checks). The melee system is PARTIAL and needs completion plus projectile support for drum towers.
**Delivers:** Centralized damage pipeline, melee formula completion, projectile (shot) objects, knockback physics, building combat, death processing.
**Addresses:** Combat System (FEATURES P1)
**Avoids:** Pitfall 4 (person state transitions during combat)

### Phase 4: Mana + Spell System + Terrain Modification
**Rationale:** Spells are the god-game identity and primary combat mechanic. They require combat (damage), effects (visuals), and buildings (targeting) to already exist. This phase also includes terrain modification since many spells modify terrain.
**Delivers:** Mana generation/costs/cooldowns, core 12 spells (Burn, Blast, Lightning, Convert Wild, Flatten, Land Bridge, Shield, Teleport, Swamp, Erosion, Earthquake, Volcano), terrain height modification with full cascade chain.
**Addresses:** Mana System, Spell System, Terrain Modification (FEATURES P1)
**Avoids:** Pitfall 5 (spell-terrain cascade)

### Phase 5: Effect System (Minimal)
**Rationale:** Spell impacts, combat hits, building fires, and construction dust all need visual feedback. Only gameplay-critical effects from the 93 types -- cosmetic/environmental effects deferred.
**Delivers:** EffectManager with pool limits, spell impact effects, death effects, construction effects, fire effects. Subset of 93 types sufficient for campaign play.
**Addresses:** Minimal Effects (FEATURES P1)
**Avoids:** Pitfall 7 (pool exhaustion -- graceful creation failure)

### Phase 6: HUD/UI + Font/Text + Menu System
**Rationale:** Independently parallelizable -- reads game state, does not mutate it. Can overlap with phases 2-5 but must be complete before AI testing (testers need to see mana, population, spell availability). Includes font/text rendering and minimum menu system.
**Delivers:** Minimap, spell bar with cooldown overlay, mana/population display, health bars, unit info panel, font rendering (3 sizes), string table, main menu, campaign select, load game screen.
**Addresses:** HUD/UI, Font/Text, Menu System (FEATURES P1)
**Avoids:** UX pitfalls (missing minimap, invisible cooldowns, no building placement feedback)

### Phase 7: AI/Scripting System
**Rationale:** The long pole. Depends on buildings, combat, spells, and person states all being functional since AI opcodes command every system. Largest single system (200+ opcodes) and the most testing-intensive. Dedicated phase because of sheer scope.
**Delivers:** Bytecode interpreter, script loading from CPSCR files, all 200+ opcodes, per-tribe personality traits, AI decision loop integrated into tick inner loop.
**Addresses:** AI/Scripting (FEATURES P1)
**Avoids:** Pitfall 3 (off-by-one, opcode gaps -- panicking stubs, disassembler-first approach)

### Phase 8: Campaign Integration
**Rationale:** Once all gameplay systems exist, wire them together into a playable campaign: victory/defeat conditions, 25-level progression, discovery system (stone head worship for unlocks), save/load, training system.
**Delivers:** Victory/defeat checks, campaign level sequence, discovery/worship system, save/load (both original format via bytemuck and new format via serde+bincode), training system (warrior/spy/preacher/super warrior).
**Addresses:** Victory/Defeat, Campaign Progression, Discovery System, Save/Load, Training System (FEATURES P1)
**Avoids:** Determinism issues verified end-to-end via golden replay tests

### Phase 9: Audio System
**Rationale:** Purely additive, zero downstream dependents. Every system generates AudioEvents but none consume audio output. Implementing audio last means the game is fully playable (silent) before any audio work begins.
**Delivers:** SDT sound bank parsing, 3D positional audio via cpal, distance attenuation matching original formula, SF2 music via rustysynth, async audio thread with lock-free command queue.
**Uses:** cpal 0.17.3, rustysynth 1.3.6, hound 3.5.1
**Avoids:** Pitfall 8 (audio blocking game loop -- async from the start)

### Phase Ordering Rationale

- **Strict dependency chain:** Object Pool -> Buildings -> Combat -> Spells -> AI. Each system depends on the previous. This order comes directly from the architecture dependency graph and cannot be reshuffled.
- **Economy before combat:** Buildings + wood + population must work before combat is meaningful (need units to fight with).
- **Combat before spells:** Every offensive spell calls the damage pipeline. Building the pipeline first means spells work immediately when added.
- **AI last among gameplay systems:** AI opcodes command every other system. Building AI before its dependencies means writing dead code that must be rewritten.
- **HUD parallel track:** UI only reads game state. It can be developed alongside phases 2-5 by a separate work stream.
- **Audio is pure polish:** No system depends on audio output. It is the only phase that can be completely skipped and still have a playable game.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 1 (Object Pool):** Needs `/gsd:research-phase` -- the UnitCoordinator migration is the riskiest refactoring in the project. Must plan the migration strategy carefully to avoid breaking 260 tests.
- **Phase 7 (AI/Scripting):** Needs `/gsd:research-phase` -- 200+ opcodes with complex operand parsing. Needs a detailed implementation plan covering the disassembler, interpreter structure, and per-opcode testing strategy.
- **Phase 4 (Spells + Terrain):** Needs `/gsd:research-phase` -- terrain modification cascade has 7 steps that must all fire correctly. Spell-terrain interaction is the most integration-heavy code in the project.

Phases with standard patterns (skip research-phase):
- **Phase 3 (Combat):** Well-documented damage formulas in RE specs. Straightforward match-on-damage-type implementation.
- **Phase 5 (Effects):** Simple lifecycle objects (spawn, animate, destroy). Pool management is the only design decision.
- **Phase 6 (HUD/UI):** Standard game UI work. No novel architectural decisions.
- **Phase 8 (Campaign Integration):** Wiring together existing systems. Victory/defeat is a 16-tick check loop. Save/load follows established patterns (bytemuck for original format, serde+bincode for new).
- **Phase 9 (Audio):** cpal is well-documented. The async command queue pattern is standard game audio architecture.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All crate versions verified via cargo search. Rationale for custom vs. library decisions is strong -- original game uses proprietary formats that no library handles. |
| Features | HIGH | Based on comprehensive 366-item inventory with DONE/PARTIAL/TODO status. Feature dependencies mapped from original binary's call graph. |
| Architecture | HIGH | Based on existing codebase analysis and reverse-engineered binary structure. The 3-layer pattern is already proven in the project. |
| Pitfalls | HIGH | Based on RE specs, original binary Ghidra analysis, and post-mortems from OpenPop (stalled Pop:TB reimplementation) and OpenRA/OpenMW (successful reimplementations). |

**Overall confidence:** HIGH

### Gaps to Address

- **UnitCoordinator migration path:** The exact strategy for migrating from `Vec<Unit>` to ObjectPool slots without breaking existing tests needs detailed planning during Phase 1 research. The 260 existing tests are the safety net.
- **AI opcode completeness:** The spec documents 200+ opcodes but some edge-case opcodes may have undocumented behavior. Plan for Frida-based tracing of the original binary to capture PC advancement for every opcode.
- **app.rs decomposition:** The render file is 3296 lines and must be split before adding new render features. This refactoring should be a pre-phase task or part of Phase 1.
- **Fixed-point vs. floating-point:** The original binary uses i16/i32 fixed-point for simulation values. The existing codebase uses f32. This mismatch may cause determinism drift over long games. Needs validation during Phase 1.
- **Constant.dat loading:** Spell costs, damage tables, and AI parameters are loaded from constant.dat in the original. Currently not parsed. Needed before Phase 4 (Spells).

## Sources

### Primary (HIGH confidence)
- Reverse engineering specs: `docs/specs/object_system.md`, `docs/specs/buildings.md`, `docs/specs/spells.md`, `docs/specs/combat_and_pathfinding.md`, `docs/specs/ai_scripting.md`, `docs/specs/water_and_effects.md`, `docs/specs/audio.md`, `docs/specs/level_save_network.md`
- Existing codebase: `src/engine/state/tick.rs`, `src/engine/state/traits.rs`, `src/engine/units/coordinator.rs`, `src/engine/command.rs`, `src/engine/frame.rs`
- Internal inventory: `things-to-implement.md` (366 items with completion status)
- cpal 0.17.3, rustysynth 1.3.6, bincode 3.0.0, serde 1.0.228, hound 3.5.1 -- all verified via cargo search

### Secondary (MEDIUM confidence)
- [OpenRA Development Goals](https://github.com/OpenRA/OpenRA/wiki/Development-Goals) -- RTS reimplementation methodology
- [OpenMW Combat Research](https://wiki.openmw.org/index.php?title=Research:Combat) -- combat system reimplementation approach
- [Are We Game Yet - Audio](https://arewegameyet.rs/ecosystem/audio/) -- Rust audio ecosystem overview
- [Gaffer on Games: Floating Point Determinism](https://gafferongames.com/post/floating_point_determinism/) -- simulation determinism pitfalls
- [Game Programming Patterns: Bytecode](https://gameprogrammingpatterns.com/bytecode.html) -- bytecode interpreter patterns

### Tertiary (LOW confidence)
- [OpenPop/OpenPopulous](https://github.com/OpenPop/OpenPopulous) -- stalled Pop:TB reimplementation (useful as cautionary example, not as technical reference)
- [Populous Reincarnated community forums](https://www.popre.net/) -- community discussion on reimplementation feasibility

---
*Research completed: 2026-03-17*
*Ready for roadmap: yes*
