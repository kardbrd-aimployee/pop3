# Roadmap: Pop3

## Overview

Pop3 transforms from a rendering tech demo into a playable 25-level Populous: The Beginning campaign. The build order follows the original binary's dependency graph: a unified object pool underpins everything, then the economy/combat gameplay loop, then god powers and visual feedback, and finally AI opponents and campaign progression. Each phase delivers a verifiable gameplay capability on top of the previous one.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Core Object System** - Unified object pool with cell grid and UnitCoordinator migration
- [x] **Phase 2: Economy and Combat** - Buildings, economy loops, person states, combat, and terrain modification
- [ ] **Phase 3: HUD and Effects** - Complete HUD (minimap, spell bar, health bars, fonts) and visual effect pool
- [ ] **Phase 4: Spell System** - 12 spells with mana costs, cooldowns, targeting, and visual feedback
- [ ] **Phase 5: AI and Campaign** - Lua AI scripting, menu system, 25-level campaign, save/load

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
- [x] 02-08-PLAN.md -- Gap closure: wire building spawn/convert/combat actions into game loop
- [x] 02-09-PLAN.md -- Gap closure: wire wood navigation and knockback into game loop
- [x] 02-10-PLAN.md -- Gap closure: ghost preview GPU rendering with alpha blending

### Phase 3: HUD and Effects
**Goal**: The player has a complete HUD showing all game state (minimap, spell bar, mana, population, health bars, info panels) and a visual effect pool that renders spell impacts, combat hits, and building events
**Depends on**: Phase 2
**Requirements**: HUD-01, HUD-02, HUD-03, HUD-04, HUD-05, HUD-06, HUD-07, HUD-08, FX-01, FX-02, FX-03, FX-04, FX-05
**Success Criteria** (what must be TRUE):
  1. The minimap shows a 128x128 overview with tribe-colored dots, camera viewport rectangle, and click-to-move camera support
  2. The spell bar shows available spells with cooldown indicators, and mana/population displays update in real time
  3. Selecting a unit or building shows an info panel with relevant stats, and health bars appear above damaged entities
  4. Text renders correctly at 12/16/24pt sizes from the original font data, with English string table loaded
  5. A 512-slot effect pool spawns visual effects for spell impacts, combat hits, deaths, and construction/destruction events
**Plans**: 5 plans

Plans:
- [ ] 03-01-PLAN.md -- String table and font data parsers (HUD-07, HUD-08)
- [ ] 03-02-PLAN.md -- Effect pool core with types and entity attachment (FX-01, FX-05)
- [ ] 03-03-PLAN.md -- HudState extensions: mana bar, population, spell cooldowns (HUD-02, HUD-03, HUD-04)
- [ ] 03-04-PLAN.md -- Minimap viewport + click-to-move, selection info panel (HUD-01, HUD-05)
- [ ] 03-05-PLAN.md -- Health bars and effect spawn wiring (HUD-06, FX-02, FX-03, FX-04)

### Phase 4: Spell System
**Goal**: The shaman can cast all 12 core spells with mana costs and cooldowns, each spell produces its documented effect, and terrain-affecting spells use the existing cascade pipeline
**Depends on**: Phase 3
**Requirements**: SPLL-01, SPLL-02, SPLL-03, SPLL-04, SPLL-05, SPLL-06, SPLL-07, SPLL-08, SPLL-09, SPLL-10, SPLL-11, SPLL-12, SPLL-13, SPLL-14
**Success Criteria** (what must be TRUE):
  1. Player can select the shaman, choose a spell from the spell bar, target a location/unit, and the spell executes with correct mana deduction and cooldown timer
  2. Offensive spells work: Burn damages a cell, Blast fires 32 projectiles in expanding ring, Lightning strikes buildings in 4 stages, Volcano launches 32 fire projectiles
  3. Terrain spells modify the landscape: Flatten levels terrain, Land Bridge raises from water, Swamp creates drowning terrain, Erosion lowers into water, Earthquake modifies 25x25 area
  4. Utility/buff spells work: Shield protects up to 8 passengers with knockback ejection, Teleport moves shaman, Convert Wild converts wild people to tribe braves
  5. Each spell has a per-type cooldown timer that prevents recasting until expired
**Plans**: TBD

Plans:
- [ ] 04-01: TBD
- [ ] 04-02: TBD
- [ ] 04-03: TBD

### Phase 5: AI and Campaign
**Goal**: AI tribes play against the human through Lua scripts, the 25-level campaign is playable from main menu to victory screen, and game state can be saved/loaded
**Depends on**: Phase 4
**Requirements**: AI-01, AI-02, AI-03, AI-04, AI-05, AI-06, AI-07, MENU-01, MENU-02, MENU-03, MENU-04, MENU-05, CAMP-01, CAMP-02, CAMP-03, CAMP-04, CAMP-05, SAVE-01, SAVE-02, SAVE-03
**Success Criteria** (what must be TRUE):
  1. AI tribes execute Lua scripts that build structures, train units, cast spells, and attack the player with difficulty scaling per level
  2. Each campaign level loads with correct objectives, and the game detects victory (all enemies eliminated) and defeat (player eliminated with reincarnation timeout)
  3. Player can progress through all 25 campaign levels, with stone head discovery unlocking new spells and buildings between levels
  4. Player can save the full game state to a file (including quicksave), load it back, and resume play with all systems restored identically
  5. Main menu provides navigation to campaign select, load game, and options, with proper transitions between screens
**Plans**: TBD

Plans:
- [ ] 05-01: TBD
- [ ] 05-02: TBD
- [ ] 05-03: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Core Object System | 3/3 | Complete    | 2026-03-18 |
| 2. Economy and Combat | 10/10 | Complete | 2026-03-18 |
| 3. HUD and Effects | 0/5 | Not started | - |
| 4. Spell System | 0/3 | Not started | - |
| 5. AI and Campaign | 0/3 | Not started | - |
