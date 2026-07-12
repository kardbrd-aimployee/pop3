# Problems

- **Two sources of truth for people:** `UnitCoordinator` keeps a `Vec<Unit>` while `ObjectPool` separately stores person data. Movement and rendering use the vector, while building, mana, and some combat paths use the pool. These representations can diverge; the pool must become the authoritative world state and rendering should use a read-only view derived from it.

- **`app.rs` mixes too many responsibilities:** Window events, GPU setup, level loading, game construction, ticking, HUD assembly, input interpretation, and rendering all live together. Split this into a simulation-facing `GameSession`/world, a renderer, an input controller, and minimal application glue.

- **The game tick is mostly an orchestration shell:** `GameWorld` has the intended update order, but most subsystem slots are `NoOp` and mana is manually bridged after the tick. The session should own concrete action, terrain, object, AI, population, mana, and victory subsystems so their order is real rather than simulated.

- **Levels are not instantiated as a complete live world:** The loader reads terrain and raw level objects, but only people become live simulation entities. Buildings and scenery remain static render objects, leaving building, resource, discovery, and damage logic disconnected from loaded levels. Introduce a `LevelDefinition -> World::from_level` path that creates every gameplay-relevant object in the canonical store.

- **Several declared game commands do nothing at runtime:** Building placement, entering buildings, and training are represented in `GameCommand`, but ignored by the running app. The HUD also presents spells without a cast action or spell system. Commands should either enqueue real domain actions or be omitted until implemented.

- **Reused `u16` object handles can become stale:** The pool reuses slot IDs without generation tracking. A destroyed target, occupant, projectile, or effect reference can silently resolve to a different newly allocated object. Use generational handles internally, or enforce complete invalidation on every destruction path.

- **The planned Lua replacement for original AI bytecode risks campaign incompatibility:** Original campaign levels ship `CPSCR` scripts. Requiring external Lua equivalents would create a second, potentially inaccurate campaign definition. Parse the original scripts into a tested internal instruction representation; Lua can remain an optional tooling or authoring layer.

- **Core simulation parameters are still placeholders:** Mana and spell costs are estimated rather than parsed from `constant.dat`. This is acceptable for a prototype but incompatible with a faithful remake. Economy, training, combat, and spell configuration need data-driven constants from the original files.

- **Gameplay effects and cosmetic effects are not clearly separated:** A separate visual effect pool is reasonable for particles, but persistent shields, spell targets, terrain changes, and hazards must be canonical gameplay state: ticked, queryable, renderable, and serializable.

- **Test coverage is module-heavy rather than game-heavy:** The unit-test suite validates many isolated components but does not prove a level can be played. Add data-backed integration tests that load an original level, instantiate live entities, build a hut, gather wood, spawn a brave, cast a spell, resolve victory/defeat, and save/load an identical continuation.

- **The current roadmap overstates runtime completeness:** Several building, HUD, effect, terrain, and victory modules are unit-tested yet not connected to the running game. Track completion using end-to-end acceptance criteria, not only the presence of modules and unit tests.
