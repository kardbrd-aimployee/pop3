# Roadmap: Pop3

## Overview

Pop3 transforms from a rendering tech demo into a playable 25-level Populous: The Beginning campaign. The build order follows the original binary's dependency graph: a unified object pool underpins everything, then the economy/combat gameplay loop, then god powers and visual feedback, and finally AI opponents and campaign progression. Each phase delivers a verifiable gameplay capability on top of the previous one.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Core Object System** - Unified object pool with cell grid and UnitCoordinator migration
- [ ] **Phase 2: Economy and Combat** - Buildings, economy loops, person states, combat, and terrain modification
- [ ] **Phase 3: Spells, Effects, and Interface** - 12 spells with mana economy, visual effects, and full HUD/UI
- [ ] **Phase 4: AI and Campaign** - Lua AI scripting, menu system, 25-level campaign, save/load

## Phase Details

### Phase 1: Core Object System
**Goal**: Every game object (person, building, effect, projectile) lives in a unified pool with spatial indexing via a modern Rust-idiomatic approach, and all 266 existing tests still pass
**Depends on**: Nothing (first phase)
**Requirements**: OBJ-01, OBJ-02, OBJ-03, OBJ-04, OBJ-05
**Success Criteria** (what must be TRUE):
  1. Objects are allocated from a 1101-capacity pool with stable handles and O(1) create/destroy (modern Rust approach, not faithful binary replica)
  2. A 128x128 cell grid tracks which objects occupy each cell, and moving an object updates its cell linkage automatically
  3. The existing UnitCoordinator uses the pool instead of owning a Vec, and all 266 existing tests pass without behavior changes
  4. Object creation, destruction, and reinitialization produce correct observable behavior (same objects exist, same spatial relationships)
**Plans**: 3 plans

Plans:
- [x] 01-01-PLAN.md -- ObjectPool with create/destroy/get and person iteration (TDD)
- [x] 01-02-PLAN.md -- Cell-based spatial grid with per-cell doubly-linked object lists (TDD)
- [x] 01-03-PLAN.md -- UnitCoordinator migration to pool-backed storage

### Phase 2: Economy and Combat
**Goal**: Players can build structures, gather wood, grow population, train units, fight with melee and projectiles, and modify terrain -- the complete gameplay loop minus spells
**Depends on**: Phase 1
**Requirements**: BLDG-01, BLDG-02, BLDG-03, BLDG-04, BLDG-05, BLDG-06, BLDG-07, BLDG-08, ECON-01, ECON-02, ECON-03, ECON-04, ECON-05, PRSN-01, PRSN-02, PRSN-03, PRSN-04, PRSN-05, PRSN-06, PRSN-07, PRSN-08, CMBT-01, CMBT-02, CMBT-03, CMBT-04, CMBT-05, TERR-01, TERR-02, TERR-03
**Success Criteria** (what must be TRUE):
  1. Player can place a building (ghost preview, terrain validation), watch it construct with wood consumption, and see braves spawn from completed huts
  2. Braves autonomously gather wood from trees, carry it back to buildings, and wood is stored and consumed for construction and training
  3. Player can send a brave into a training building and it converts to warrior/spy/preacher/super warrior after the correct timer with mana/wood costs deducted
  4. Units engage in melee combat with correct damage formulas, drum towers fire projectiles, knockback physics work, and dead units are cleaned up properly
  5. Terrain can be raised/lowered with the full cascade (normals, walkability, water, pathfinding, mesh) updating correctly
**Plans**: 10 plans

Plans:
- [x] 02-01-PLAN.md -- Building data foundation: BuildingData, state machine, occupants, pool integration
- [ ] 02-02-PLAN.md -- Terrain modification with height change and cascade pipeline
- [x] 02-03-PLAN.md -- Economy module: mana generation, population capacity, wood costs
- [x] 02-04-PLAN.md -- Person state extensions: 8 new states (building, training, wood, combat)
- [x] 02-05-PLAN.md -- Building behaviors: spawning, training, placement, damage, combat
- [x] 02-06-PLAN.md -- Combat subsystem: projectiles, knockback, damage, death, drum towers
- [x] 02-07-PLAN.md -- Integration: wire subsystems into game loop, wood gathering, FrameState
- [ ] 02-08-PLAN.md -- Gap closure: wire building spawn/convert/combat actions into game loop
- [ ] 02-09-PLAN.md -- Gap closure: wire wood navigation and knockback into game loop
- [ ] 02-10-PLAN.md -- Gap closure: ghost preview GPU rendering with alpha blending

### Phase 3: Spells, Effects, and Interface
**Goal**: The shaman can cast 12 spells with mana costs and cooldowns, spell impacts produce visual effects, and the player has a complete HUD showing all game state
**Depends on**: Phase 2
**Requirements**: SPLL-01, SPLL-02, SPLL-03, SPLL-04, SPLL-05, SPLL-06, SPLL-07, SPLL-08, SPLL-09, SPLL-10, SPLL-11, SPLL-12, SPLL-13, SPLL-14, FX-01, FX-02, FX-03, FX-04, FX-05, HUD-01, HUD-02, HUD-03, HUD-04, HUD-05, HUD-06, HUD-07, HUD-08
**Success Criteria** (what must be TRUE):
  1. Player can select the shaman, choose a spell from the spell bar, target a location/unit, and the spell executes with correct mana deduction and cooldown timer
  2. All 12 core spells produce their documented effects: Burn damages a cell, Blast fires 32 projectiles, Lightning strikes buildings in 4 stages, terrain spells (Flatten, Land Bridge, Swamp, Erosion, Earthquake) modify the landscape, Volcano launches fire projectiles, Shield protects units, Teleport moves shaman, Convert Wild converts wild people
  3. Spell impacts, combat hits, deaths, and construction/destruction produce visible effects from a 512-slot effect pool
  4. The minimap shows a 128x128 overview with tribe-colored dots, the spell bar shows available spells with cooldown indicators, and mana/population displays update in real time
  5. Selecting a unit or building shows an info panel, health bars appear above damaged entities, and all text renders correctly at 12/16/24pt sizes from the original font data
**Plans**: TBD

Plans:
- [ ] 03-01: TBD
- [ ] 03-02: TBD
- [ ] 03-03: TBD

### Phase 4: AI and Campaign
**Goal**: AI tribes play against the human through Lua scripts, the 25-level campaign is playable from main menu to victory screen, and game state can be saved/loaded
**Depends on**: Phase 3
**Requirements**: AI-01, AI-02, AI-03, AI-04, AI-05, AI-06, AI-07, MENU-01, MENU-02, MENU-03, MENU-04, MENU-05, CAMP-01, CAMP-02, CAMP-03, CAMP-04, CAMP-05, SAVE-01, SAVE-02, SAVE-03
**Success Criteria** (what must be TRUE):
  1. AI tribes execute Lua scripts that build structures, train units, cast spells, and attack the player with difficulty scaling per level
  2. Each campaign level loads with correct objectives, and the game detects victory (all enemies eliminated) and defeat (player eliminated with reincarnation timeout)
  3. Player can progress through all 25 campaign levels, with stone head discovery unlocking new spells and buildings between levels
  4. Player can save the full game state to a file (including quicksave), load it back, and resume play with all systems restored identically
  5. Main menu provides navigation to campaign select, load game, and options, with proper transitions between screens
**Plans**: TBD

Plans:
- [ ] 04-01: TBD
- [ ] 04-02: TBD
- [ ] 04-03: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Core Object System | 3/3 | Complete    | 2026-03-18 |
| 2. Economy and Combat | 9/10 | In Progress|  |
| 3. Spells, Effects, and Interface | 0/3 | Not started | - |
| 4. AI and Campaign | 0/3 | Not started | - |
