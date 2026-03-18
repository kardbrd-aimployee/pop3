# Pitfalls Research

**Domain:** Classic RTS/god-game reimplementation (Populous: The Beginning)
**Researched:** 2026-03-17
**Confidence:** HIGH (based on RE specs, codebase analysis, and open-source reimplementation post-mortems)

## Critical Pitfalls

### Pitfall 1: Object Pool Allocation Order Breaks Determinism

**What goes wrong:**
The original game uses a fixed-size object pool (1101 active objects) with separate high-priority (units/buildings) and low-priority (effects/particles) free lists. If the reimplementation allocates objects from the pool in a different order than the original binary -- even slightly -- every subsequent object gets a different index. Since the AI scripting system, combat targeting, and spell effects all reference objects by pool index, a single allocation-order divergence cascades into completely different game behavior. The RNG is already faithfully reimplemented (`GameRng` matches the original seed), but determinism is meaningless if the RNG is called in a different order due to objects existing at different pool slots.

**Why it happens:**
Developers implement the pool as a simple `Vec` or `HashMap` without replicating the original's linked-list free-list structure. Rust's `HashMap` iteration order is non-deterministic. Even a `Vec`-based pool with "first free slot" allocation diverges if the free list isn't maintained identically to the original's two-tier (high/low priority) system documented at `DAT_008788b4` and `DAT_008788b8`.

**How to avoid:**
- Implement the object pool as a fixed-size array with explicit linked-list free lists, exactly mirroring the original's structure (see `docs/specs/object_system.md`).
- The pool must have exactly 1101 slots split into high-priority and low-priority regions.
- `Object_Create` must pull from the correct free list based on model type.
- Write fixture tests: capture object creation sequences from the original binary (via Frida) and verify the reimplementation produces identical pool indices.

**Warning signs:**
- Units spawned in different positions than the original on the same level.
- AI behaves differently from the original despite identical scripts.
- Effects or projectiles appear at wrong locations or timings.
- Save files from the reimplementation produce different checksums than expected.

**Phase to address:**
Core Object System (must be the first system built -- everything else depends on it).

---

### Pitfall 2: Tick Subsystem Ordering Divergence

**What goes wrong:**
The original binary's `Game_SimulationTick` (0x004bb5a0) processes subsystems in a specific order: movement, then combat, then spells, then buildings, then AI, then effects, then cleanup. The existing `GameWorld` in `tick.rs` establishes this pattern, but as new subsystems are added, developers insert them at convenient points rather than matching the original order. Since each subsystem mutates shared state (positions, health, flags), processing them in the wrong order produces subtly different outcomes. A unit that should die from combat damage before the spell system runs might instead survive and cast a spell, changing the entire game state.

**Why it happens:**
When building incrementally, each new system gets appended to the tick loop. The developer tests it in isolation and it "works." But the interaction order matters: the original binary's tick order was designed (or evolved) to handle specific edge cases. Moving building population growth before combat resolution, for example, means a building that should be destroyed first instead spawns an extra unit.

**How to avoid:**
- The tick order is already documented in the RE specs. Treat it as a contract, not a guideline.
- Add a comment block at the top of the tick function listing the canonical order with original binary addresses.
- Never reorder subsystem calls without a documented reason and regression testing.
- Create "golden replay" tests: record input sequences on the original binary, replay them in the reimplementation, compare game state snapshots at each tick.

**Warning signs:**
- "Off by one tick" bugs where events happen one frame too early or late.
- Units dying in different orders than the original.
- Buildings completing construction at different tick counts.

**Phase to address:**
Core Object System and every subsequent phase. Each new subsystem must be inserted at the correct position in the tick loop.

---

### Pitfall 3: AI Bytecode Interpreter Off-by-One and Opcode Gaps

**What goes wrong:**
The AI scripting system uses 200+ opcodes (16-bit, starting at base 0x404) with a custom bytecode interpreter. Reimplementing this interpreter has two catastrophic failure modes: (1) getting the program counter advancement wrong for even one opcode, causing all subsequent script execution to read garbage; (2) leaving opcodes unimplemented, which silently skips AI decisions and makes computer players appear brain-dead.

The OpenPop project (C++ reimplementation of Pop:TB) stalled as "Unplayable" partly because the complexity of faithfully reproducing the AI scripting system was underestimated. The 200+ opcodes interact with nearly every other game system (building, training, spell casting, targeting), making the interpreter a dependency bottleneck.

**Why it happens:**
Bytecode interpreters are tedious to implement. Each opcode has different operand sizes and side effects. Developers implement the "important" opcodes first and stub the rest, but the original AI scripts use obscure opcodes in specific levels. Level 15 might work fine while level 16 crashes because it uses an opcode that was left as a no-op. Additionally, the three value types (immediate, variable reference, internal attribute lookup) each have different byte widths, and mishandling one corrupts the program counter.

**How to avoid:**
- Implement ALL opcodes before testing any AI behavior. Stubs should panic with the opcode number, not silently succeed.
- Write a bytecode disassembler first (before the interpreter) to verify script parsing.
- Extract original script files and create round-trip tests: disassemble then reassemble and compare bytes.
- For program counter advancement, trace the original binary's `AI_ExecuteScriptCommand` with Frida to capture (opcode, PC_before, PC_after) tuples for each script.
- Test against every campaign level's AI script, not just the first few.

**Warning signs:**
- AI tribes that never build, never attack, or never cast spells.
- Scripts that work on early levels but crash on later ones.
- AI that behaves identically regardless of game state (stuck in a loop).
- Attribute lookups (Type 2 values) returning 0 for everything.

**Phase to address:**
AI/Scripting System phase. This should be a dedicated phase, not combined with other systems, because of the sheer opcode count and testing burden.

---

### Pitfall 4: Person State Machine Incomplete Transitions

**What goes wrong:**
The person state machine has 40+ states (`Person_SetState` at 0x004fd5d0) with complex transition rules. The existing implementation covers basic states (idle, move, goto, wander, attack basics). When adding combat, spells, building interaction, and training, developers implement the new states but miss transition edges -- for example, a unit in "entering building" state that takes damage should transition to "hit react" then back to "entering building," but the reimplementation transitions to "idle" instead, leaving the unit stranded outside the building.

**Why it happens:**
The original binary's `Person_SetState` is a massive switch statement where many cases fall through or call other state transitions conditionally. The Ghidra decompilation shows the happy path clearly but obscures edge-case transitions hidden in nested conditionals. Developers implement what they see in the decompilation without testing every possible state-to-state pair.

**How to avoid:**
- Build a complete state transition matrix (40x40) from the RE specs before writing code.
- For each state, document: entry conditions, exit conditions, interrupt conditions (damage, spell, drowning).
- Implement an "unexpected state transition" assertion that logs when a transition occurs that isn't in the matrix.
- Test state transitions by capturing person state logs from the original binary for entire levels.

**Warning signs:**
- Units that freeze (stuck in a state with no valid exit transition).
- Units that teleport (state resets position to origin).
- Units inside buildings that can't exit.
- Combat units that stop fighting mid-battle and stand idle.

**Phase to address:**
Building System and Combat System phases must both update the state transition matrix. Every phase that adds new person states must verify it hasn't broken existing transitions.

---

### Pitfall 5: Spell-Terrain Interaction Cascade Failures

**What goes wrong:**
Many spells modify terrain (Earthquake, Volcano, Erosion, Swamp, Land Bridge, Flatten). Terrain modification triggers a cascade of side effects: walkability recalculation, building stability checks (buildings on modified terrain may sink or collapse), water level changes, pathfinding graph invalidation, and cell flag updates. Missing any link in this cascade causes desyncs: a building appears to stand on flat ground visually but the walkability system thinks it's underwater, or pathfinding routes through newly raised land that the cell flags still mark as blocked.

**Why it happens:**
Each system (terrain, walkability, pathfinding, buildings, water) is built as an independent module. The developer implements Earthquake to modify heightmap values and it "looks correct" visually. But they forget to call `Region_UpdateAfterTerrainChange` (or its equivalent), so the pathfinding graph becomes stale. The existing shore erosion pass (already implemented) shows this pattern -- it modifies walkability but is a separate pass from the heightmap update.

**How to avoid:**
- Document the complete cascade chain for terrain modification from the original binary:
  1. Modify heightmap cells
  2. Recalculate cell normal vectors
  3. Update walkability flags
  4. Check building foundations (sink/destroy if invalid)
  5. Update water intersection
  6. Invalidate pathfinding regions
  7. Rebuild affected mesh chunks
- Implement terrain modification as a single function that performs ALL cascade steps, not as separate calls that the spell code must remember to invoke.
- Test with the Volcano spell (most destructive terrain modifier) -- if it works correctly, simpler spells will too.

**Warning signs:**
- Units walking through newly raised terrain.
- Buildings floating above or sinking into modified terrain.
- Pathfinding routes ignoring terrain changes (units take old paths).
- Water not filling lowered terrain.

**Phase to address:**
Terrain System phase must establish the cascade function. Spell System phase must use it exclusively and never modify terrain directly.

---

### Pitfall 6: Building Occupant Tracking Desyncs from Object Pool

**What goes wrong:**
Buildings store references to their occupants (people inside training, garrisoned in towers, or housed in tepees). These references are object pool indices. When occupants die (from spells, building destruction, or drowning), the building's occupant list must be updated. If the building still references a destroyed object's pool slot, and that slot gets reallocated to a new object, the building now "owns" a random unit from a different tribe. This causes population count corruption, ghost units, and cascading crashes.

**Why it happens:**
The original binary handles this through careful destroy-order in the tick loop and explicit occupant-ejection on building destruction (`Building_EjectPerson` at 0x00432800, `Building_OnDestroy` at 0x00433bb0). Reimplementations often skip the ejection step or handle it after the object pool has already recycled the slot. Rust's ownership system helps prevent dangling pointers but not semantic reference corruption (a valid index pointing to the wrong object).

**How to avoid:**
- Use generation counters on pool slots: each slot has a generation number that increments on deallocation. References store (index, generation) pairs. Stale references are detected by generation mismatch.
- Mirror the original's destroy order exactly: eject occupants BEFORE destroying the building object.
- When destroying a person, remove them from their building's occupant list BEFORE returning their slot to the free list.
- Test building destruction under combat: destroy a building full of training warriors while enemies are attacking.

**Warning signs:**
- Population count drifting from actual living units.
- Units appearing inside enemy buildings.
- Crash or panic when iterating a building's occupant list.
- "Phantom" units that exist in building occupancy but not in the world.

**Phase to address:**
Building System phase. The occupant tracking design must be established before Combat System adds destruction scenarios.

---

### Pitfall 7: Effect System Object Pool Exhaustion

**What goes wrong:**
The original game's low-priority pool holds 640 effect/particle objects. Spells like Firestorm, Lightning Storm, and Volcano can create dozens of effects simultaneously. If the reimplementation doesn't enforce the pool limit, a single Firestorm spell in a crowded area can attempt to create hundreds of fire particles, exhausting the pool and preventing other game objects (shots, spell markers) from being created. The original binary silently fails `Object_Create` when the pool is full -- the reimplementation might panic instead.

**Why it happens:**
Developers test spells in isolation on empty maps. A single Blast works fine. But in actual gameplay, multiple AI tribes casting spells simultaneously while buildings burn and projectiles fly can easily hit the 640-object low-priority ceiling. The original binary's graceful degradation (silent creation failure) is a design choice, not a bug.

**How to avoid:**
- `Object_Create` must return `Option<ObjectIndex>` (or equivalent), never panic on pool exhaustion.
- All callers must handle the `None` case gracefully (skip effect creation, not crash).
- Add a pool utilization metric visible in debug mode.
- Stress test with maximum-chaos scenarios: 4 tribes, all casting spells, multiple buildings burning.

**Warning signs:**
- Panics during intense spell battles.
- Frame rate drops from unbounded effect creation (if pool limit isn't enforced).
- Missing visual effects during busy battles (this is actually correct behavior if pool is full).

**Phase to address:**
Core Object System phase (pool design). Effect System phase (graceful handling of creation failure).

---

### Pitfall 8: Audio System Blocking the Game Loop

**What goes wrong:**
The audio system (3D positional audio, SFX, music via SoundFont) is added late in development. Developers integrate it synchronously into the tick loop: "play sound" calls block until the audio backend acknowledges them. On some systems, audio initialization takes 100+ ms, and individual play calls can stall for 5-10 ms if the audio thread is busy. This makes the game stutter during intense battles when dozens of sounds fire per tick.

**Why it happens:**
Audio seems simple: "just call play_sound()." But audio backends (CPAL, rodio, kira) have their own threading models. Calling them from the game tick thread without buffering creates contention. The original binary used DirectSound with a callback model that naturally decoupled audio from the game loop.

**How to avoid:**
- Design the audio system as a separate thread with a lock-free command queue from the start.
- The game tick loop enqueues sound commands (play, stop, set position); the audio thread processes them asynchronously.
- Never block the game tick on audio operations.
- Use `kira` or `oddio` which are designed for game audio with built-in command queues.
- If audio initialization fails, the game must still run (audio is optional).

**Warning signs:**
- Frame rate drops when many sounds play simultaneously.
- Game stutter at level start (audio loading).
- Entire game freezes if audio device is disconnected.

**Phase to address:**
Audio System phase. Design the async architecture before writing any audio code.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Stuffing new systems into `app.rs` | Fast to add, no module design needed | Already 3296 lines; adding buildings, spells, AI makes it unmaintainable | Never -- extract `GameEngine` from `app.rs` before adding systems |
| Using `f32` for game simulation values | Easier math, matches Rust conventions | Original uses `i16`/`i32` fixed-point; floating-point accumulation errors break determinism over long games | Never for simulation state; acceptable for rendering-only math |
| Hardcoding spell parameters | Faster initial implementation | Original loads from `constant.dat`; hardcoding prevents tuning and breaks campaign balance | Only for initial testing with TODO to load from data files |
| Implementing AI opcodes as no-ops | Get basic AI "running" quickly | Silent behavior divergence; hard to debug why AI doesn't attack on level 12 | Never -- use panicking stubs instead |
| Skipping occupant ejection on building destroy | Simpler destruction logic | Ghost references, population desync, potential crashes | Never |
| Per-frame vertex buffer rebuilds for effects | Simple rendering pipeline | Already identified as a bottleneck with current unit counts; effects multiply this | Acceptable for MVP, must fix before spell effects phase |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Linear scan for spell target finding | Spells take 10+ ms to find targets in dense areas | Use the cell-based spatial grid (128x128) that the original binary uses | > 500 units on map with area spells |
| Rebuilding entire pathfinding graph on terrain change | Terrain-modifying spells cause multi-second hitches | Only invalidate affected regions (25x25 cell area per the Effect_Update spec) | Any terrain-modifying spell in gameplay |
| Unbounded effect particle creation | Frame drops to single digits during Firestorm + Volcano combo | Enforce the 640-object low-priority pool limit | 2+ area spells active simultaneously |
| Synchronous SoundFont loading for music | 500ms+ stall on level transition | Load SoundFont async at startup, not on first music play | Every level load |
| Brute-force collision for projectiles | Shot_ProcessImpact iterating all objects per projectile | Use cell grid: projectiles only check objects in impact cell and neighbors | > 20 simultaneous projectiles (common in 4-tribe battles) |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Spell effects without audio feedback | Spells feel weightless; players unsure if spell landed | Audio and visual effects must ship together, never visual-only |
| AI that doesn't attack on schedule | Players think game is broken when AI sits idle due to unimplemented opcodes | Implement AI scripting before exposing campaign mode; stub levels are worse than no levels |
| Building placement without visual feedback | Players place buildings in invalid locations, nothing happens | Show ghost building with red/green validity indicator matching original UX |
| Missing minimap during combat | Players lose track of battles across the map | Minimap is table-stakes for RTS; implement before combat system |
| Spell cooldown timers not visible | Players spam-click spells wondering why nothing happens | HUD spell bar with cooldown overlay must exist before spell system is playable |

## "Looks Done But Isn't" Checklist

- [ ] **Object Pool:** Often missing generation counters on slots -- verify that stale references are detected, not silently resolved to wrong objects
- [ ] **Building Construction:** Often missing the "must have enough flat terrain" check -- verify buildings can't be placed on slopes or water
- [ ] **Combat Damage:** Often missing the shield damage reduction shift -- verify shielded units take reduced damage, not full
- [ ] **Spell Mana Cost:** Often missing altitude band bonus -- verify spells cast from high ground cost less / do more damage
- [ ] **AI Scripts:** Often missing Type 2 attribute evaluation (internal state lookups) -- verify AI can query its own tribe statistics
- [ ] **Person State Machine:** Often missing the "drowning" interrupt -- verify units in any state transition to drowning when water rises
- [ ] **Building Destruction:** Often missing occupant ejection -- verify people inside are ejected or killed, not leaked
- [ ] **Effect Cleanup:** Often missing the state 5 (Complete) handler -- verify effects destroy themselves and return pool slots
- [ ] **Terrain Modification:** Often missing pathfinding region invalidation -- verify units recalculate paths after Earthquake
- [ ] **Save/Load:** Often missing object pool state -- verify that saving mid-game and loading produces identical continuation

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Object pool allocation order wrong | HIGH | Must redesign pool to match original's free-list structure; all object indices change, breaking existing tests |
| Tick subsystem ordering wrong | MEDIUM | Reorder calls in tick function; run golden replay tests to verify; may need to fix cascading state differences |
| AI opcode off-by-one | HIGH | Must trace original binary to find divergence point; every script execution after the bug is invalid |
| State machine missing transitions | LOW | Add missing transitions one at a time; each is independent |
| Terrain cascade incomplete | MEDIUM | Add missing cascade steps to the terrain modification function; retest all terrain-modifying spells |
| Building occupant desync | HIGH | Must audit all code paths that destroy persons or buildings; add generation counters retroactively |
| Effect pool exhaustion panic | LOW | Change `Object_Create` return type to `Option`; grep for all callers and add `None` handling |
| Audio blocking game loop | MEDIUM | Redesign to async command queue; requires touching all play_sound call sites |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Object pool allocation order | Core Object System | Fixture tests comparing pool indices with original binary traces |
| Tick subsystem ordering | Core Object System + every subsequent phase | Golden replay tests: same inputs produce same outputs |
| AI bytecode interpreter | AI/Scripting System (dedicated phase) | Run all 25 campaign scripts; compare AI decisions with original |
| Person state machine transitions | Building System, Combat System | State transition matrix coverage: every valid pair tested |
| Spell-terrain cascade | Terrain System, then Spell System | Volcano test: verify heightmap, walkability, pathfinding, buildings all update |
| Building occupant tracking | Building System | Destroy-under-load test: kill buildings full of units during combat |
| Effect pool exhaustion | Core Object System, Effect System | Stress test: 4 tribes casting simultaneous spells |
| Audio blocking | Audio System | Measure tick duration with and without audio; must be < 1ms difference |

## Sources

- [OpenPop - Populous: The Beginning reimplementation (stalled)](https://github.com/OpenPop/OpenPopulous) -- demonstrates the difficulty of faithful Pop:TB reimplementation
- [OpenPop Notes - Populous resource documentation](https://github.com/OpenPop/notes) -- community RE knowledge
- [Populous Reincarnated community](https://www.popre.net/forum/populous-discussion-f4/a-whole-new-open-source-version-freepop-t1418.html) -- post-mortem discussions on reimplementation attempts
- [OpenMW Combat Research](https://wiki.openmw.org/index.php?title=Research:Combat) -- combat reimplementation methodology from a successful project
- [OpenRA Architecture (Delft University analysis)](https://delftswa.github.io/chapters/openra/) -- component-based entity system lessons from RTS reimplementation
- [Gaffer on Games: Floating Point Determinism](https://gafferongames.com/post/floating_point_determinism/) -- determinism pitfalls in game simulation
- [Game Programming Patterns: Bytecode](https://gameprogrammingpatterns.com/bytecode.html) -- bytecode interpreter implementation patterns and pitfalls
- [Box2D Determinism](https://box2d.org/posts/2024/08/determinism/) -- contact ordering and iteration-order determinism
- [hrttf111/pop3-rev](https://github.com/hrttf111/pop3-rev) -- parallel Populous: The Beginning RE effort with Ghidra
- Project RE specs in `docs/specs/` -- primary source for original binary behavior
- Project codebase analysis in `.planning/codebase/CONCERNS.md` -- existing known issues

---
*Pitfalls research for: Populous: The Beginning reimplementation -- gameplay systems phase*
*Researched: 2026-03-17*
