# Requirements: Pop3

**Defined:** 2026-03-17
**Core Value:** Faithful reproduction of the original Populous: The Beginning gameplay on modern platforms

## v1 Requirements

Requirements for playable single-player campaign (25 levels).

### Core Object System

- [x] **OBJ-01**: Unified object storage supporting all 11 model types with stable handles, at least 1101 capacity (modern Rust-idiomatic approach, not faithful replica of original binary's allocation system)
- [x] **OBJ-02**: Object lifecycle (create, destroy, reinitialize) with correct game logic behavior (observable equivalence, not allocation-order faithfulness)
- [x] **OBJ-03**: Cell-based spatial grid (128x128, 16 bytes/cell) with per-cell object linked lists
- [x] **OBJ-04**: Object position updates that maintain cell linkage (Object_SetPosition)
- [x] **OBJ-05**: UnitCoordinator migration to borrow from unified pool instead of owning Vec<Unit>

### Building System

- [x] **BLDG-01**: Building state machine (init, construction, active, destroying, sinking, teardown)
- [x] **BLDG-02**: Building construction with progress animation and wood consumption
- [ ] **BLDG-03**: Building placement UI with ghost preview and terrain validation
- [x] **BLDG-04**: Occupant system (6 slots per building, enter/exit, capacity checks)
- [ ] **BLDG-05**: Population growth from huts (spawning braves at HUT_SPROG_TIME rates)
- [ ] **BLDG-06**: Training conversion (brave to warrior/spy/preacher/super warrior) with wood/mana costs
- [ ] **BLDG-07**: Building damage and destruction with debris and chain damage
- [ ] **BLDG-08**: Building combat (6 fighter slots, attack types, occupant fighting)

### Economy

- [ ] **ECON-01**: Wood gathering (brave walks to tree, chops, carries wood back)
- [x] **ECON-02**: Wood storage in buildings with consumption tracking
- [x] **ECON-03**: Mana generation per unit type and activity (housed, training, idle, working)
- [x] **ECON-04**: Mana pool per tribe with MAX_MANA cap
- [x] **ECON-05**: Population cap based on housing capacity (hut levels 1-3)

### Combat

- [x] **CMBT-01**: Complete melee damage formula (FIGHT_DAMAGE[subtype] * health / max_health, min 32)
- [x] **CMBT-02**: Projectile system (shot types, tracking, AOE impact, knockback)
- [x] **CMBT-03**: Drum tower auto-attack with projectiles
- [x] **CMBT-04**: Death states with proper cleanup and kill tracking
- [x] **CMBT-05**: Knockback physics (angle-based velocity from Combat_ApplyKnockback)

### Spell System

- [ ] **SPLL-01**: Spell casting framework (targeting, validation, mana cost, cooldown)
- [ ] **SPLL-02**: Burn spell (single cell fire, 15-25 HP damage)
- [ ] **SPLL-03**: Blast spell (32 projectiles expanding ring)
- [ ] **SPLL-04**: Lightning Bolt (4-stage state machine, targets buildings)
- [ ] **SPLL-05**: Convert Wild (convert wild person to tribe brave)
- [ ] **SPLL-06**: Flatten Land (flatten terrain area)
- [ ] **SPLL-07**: Land Bridge (raise terrain from water)
- [ ] **SPLL-08**: Shield (protect up to 8 passengers, knockback physics)
- [ ] **SPLL-09**: Teleport (shaman teleportation)
- [ ] **SPLL-10**: Swamp (create drowning terrain)
- [ ] **SPLL-11**: Erosion (lower terrain into water)
- [ ] **SPLL-12**: Earthquake (25x25 cell height modification)
- [ ] **SPLL-13**: Volcano (32 fire projectiles)
- [ ] **SPLL-14**: Spell cooldown timers per spell type

### Terrain Modification

- [x] **TERR-01**: Height modification function (Terrain_ModifyHeight) with gradual change
- [x] **TERR-02**: Terrain cascade after modification (heights -> normals -> walkability -> buildings -> water -> pathfinding -> mesh)
- [x] **TERR-03**: Dynamic water level interaction (cells become water/land based on height)

### Person State Machine

- [ ] **PRSN-01**: Enter building state (walk into building, become occupant)
- [ ] **PRSN-02**: Exit building state (walk out, facing direction)
- [ ] **PRSN-03**: Housed state (inside housing, contributes to population)
- [ ] **PRSN-04**: Training state (in training building, conversion timer)
- [ ] **PRSN-05**: Gather wood state (walk to tree, chop, carry back)
- [ ] **PRSN-06**: Drown state (drowning in water)
- [ ] **PRSN-07**: Guard state (hold position)
- [ ] **PRSN-08**: Death effects (proper death state with cleanup)

### AI/Scripting

- [ ] **AI-01**: AI scripting engine -- Lua-based interpreter (community has documented Lua equivalents of original bytecode scripts) instead of raw bytecode VM
- [ ] **AI-02**: Script flow control (IF/ELSE/ENDIF, EVERY/DO loops, subroutine calls)
- [ ] **AI-03**: Script value types (literal, variable, internal attribute 1000-1237)
- [ ] **AI-04**: AI decision making (target selection scoring, threat assessment)
- [ ] **AI-05**: AI building placement (7-state placement machine)
- [ ] **AI-06**: Shaman command system (8 command types, 10 slots per tribe)
- [ ] **AI-07**: Difficulty scaling (separate mana/training costs for AI vs human)

### HUD/UI

- [ ] **HUD-01**: Minimap rendering (128x128, tribe-colored unit dots)
- [ ] **HUD-02**: Spell bar with available spells and cooldown indicators
- [ ] **HUD-03**: Mana bar display
- [ ] **HUD-04**: Population display
- [ ] **HUD-05**: Unit/building info panel on selection
- [ ] **HUD-06**: Health bars above units and buildings
- [ ] **HUD-07**: Font loading and text rendering (12/16/24pt)
- [ ] **HUD-08**: String table loading (English, 0x526 strings)

### Menu System

- [ ] **MENU-01**: Main menu with campaign/load/options navigation
- [ ] **MENU-02**: Campaign level select screen
- [ ] **MENU-03**: Load game screen
- [ ] **MENU-04**: Options/settings screen
- [ ] **MENU-05**: Menu button system with transitions

### Campaign

- [ ] **CAMP-01**: Victory conditions (all enemies eliminated)
- [ ] **CAMP-02**: Defeat conditions (player eliminated, reincarnation timer)
- [ ] **CAMP-03**: Campaign progression (25-level sequence, completion flags)
- [ ] **CAMP-04**: Discovery system (stone head worship for spell/building unlocks)
- [ ] **CAMP-05**: Level objectives loading (OBJECTIV.DAT)

### Save/Load

- [ ] **SAVE-01**: Save full game state to file (860KB state)
- [ ] **SAVE-02**: Load game state and restore all systems
- [ ] **SAVE-03**: Quicksave support (slot 99)

### Effects (Minimal)

- [ ] **FX-01**: Effect pool (512 max, 64 bytes per effect)
- [ ] **FX-02**: Spell impact visual effects (burn, blast, lightning)
- [ ] **FX-03**: Death/combat effects (blood, hit sparks)
- [ ] **FX-04**: Construction/destruction building effects
- [ ] **FX-05**: Effect attachment to moving objects

## v2 Requirements

Deferred to post-campaign. Tracked but not in current roadmap.

### Remaining Spells

- **SPLL-15**: Whirlwind (expanding circular wave)
- **SPLL-16**: Insect Plague (16 swarm particles)
- **SPLL-17**: Invisibility
- **SPLL-18**: Hypnotism (mind control)
- **SPLL-19**: Firestorm (meteor fireballs)
- **SPLL-20**: Ghost Army (ghost duplicates)
- **SPLL-21**: Angel of Death (powerful flying unit)
- **SPLL-22**: Blood Lust (damage multiplier)
- **SPLL-23**: Armageddon (final battle)

### Audio

- **AUD-01**: 3D positional audio with distance attenuation
- **AUD-02**: Sound effects from SDT files
- **AUD-03**: SoundFont music playback (popfight.sf2)
- **AUD-04**: Ambient drone sounds

### Vehicles

- **VEH-01**: Boat construction from boat huts
- **VEH-02**: Airship construction from air huts
- **VEH-03**: Unit boarding/disembarking
- **VEH-04**: Vehicle movement and physics

### Other

- **CRTR-01**: Creature AI (bears, wolves, eagles)
- **SPY-01**: Spy disguise and sabotage abilities
- **PRCH-01**: Preacher conversion ability
- **FX-06**: Full 93-type effect system (environmental, particle, weather)
- **SCEN-01**: Scenery interaction (tree burning, portals)

## Out of Scope

| Feature | Reason |
|---------|--------|
| Multiplayer/Networking | Massive complexity (lockstep sync, lobby, desync detection); complete single-player first |
| Multi-language support (CJK) | English-only for v1; string table infrastructure enables later localization |
| Custom key bindings | 207-action binding system; hardcoded defaults sufficient for v1 |
| Tutorial levels | Custom scripting; target audience knows the game |
| Modding support | Not in original game scope |
| Replay system | Nice to have; deterministic RNG exists as foundation for later |
| Debug/cheat system | Use Rust debug builds instead |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| OBJ-01 | Phase 1 | Complete |
| OBJ-02 | Phase 1 | Complete |
| OBJ-03 | Phase 1 | Complete (01-02) |
| OBJ-04 | Phase 1 | Complete (01-02) |
| OBJ-05 | Phase 1 | Pending |
| BLDG-01 | Phase 2 | Complete (02-01) |
| BLDG-02 | Phase 2 | Complete (02-01) |
| BLDG-03 | Phase 2 | Pending |
| BLDG-04 | Phase 2 | Complete (02-01) |
| BLDG-05 | Phase 2 | Pending |
| BLDG-06 | Phase 2 | Pending |
| BLDG-07 | Phase 2 | Pending |
| BLDG-08 | Phase 2 | Pending |
| ECON-01 | Phase 2 | Pending |
| ECON-02 | Phase 2 | Complete |
| ECON-03 | Phase 2 | Complete |
| ECON-04 | Phase 2 | Complete |
| ECON-05 | Phase 2 | Complete |
| PRSN-01 | Phase 2 | Pending |
| PRSN-02 | Phase 2 | Pending |
| PRSN-03 | Phase 2 | Pending |
| PRSN-04 | Phase 2 | Pending |
| PRSN-05 | Phase 2 | Pending |
| PRSN-06 | Phase 2 | Pending |
| PRSN-07 | Phase 2 | Pending |
| PRSN-08 | Phase 2 | Pending |
| CMBT-01 | Phase 2 | Complete |
| CMBT-02 | Phase 2 | Complete |
| CMBT-03 | Phase 2 | Complete |
| CMBT-04 | Phase 2 | Complete |
| CMBT-05 | Phase 2 | Complete |
| TERR-01 | Phase 2 | Complete |
| TERR-02 | Phase 2 | Complete |
| TERR-03 | Phase 2 | Complete |
| SPLL-01 | Phase 3 | Pending |
| SPLL-02 | Phase 3 | Pending |
| SPLL-03 | Phase 3 | Pending |
| SPLL-04 | Phase 3 | Pending |
| SPLL-05 | Phase 3 | Pending |
| SPLL-06 | Phase 3 | Pending |
| SPLL-07 | Phase 3 | Pending |
| SPLL-08 | Phase 3 | Pending |
| SPLL-09 | Phase 3 | Pending |
| SPLL-10 | Phase 3 | Pending |
| SPLL-11 | Phase 3 | Pending |
| SPLL-12 | Phase 3 | Pending |
| SPLL-13 | Phase 3 | Pending |
| SPLL-14 | Phase 3 | Pending |
| FX-01 | Phase 3 | Pending |
| FX-02 | Phase 3 | Pending |
| FX-03 | Phase 3 | Pending |
| FX-04 | Phase 3 | Pending |
| FX-05 | Phase 3 | Pending |
| HUD-01 | Phase 3 | Pending |
| HUD-02 | Phase 3 | Pending |
| HUD-03 | Phase 3 | Pending |
| HUD-04 | Phase 3 | Pending |
| HUD-05 | Phase 3 | Pending |
| HUD-06 | Phase 3 | Pending |
| HUD-07 | Phase 3 | Pending |
| HUD-08 | Phase 3 | Pending |
| AI-01 | Phase 4 | Pending |
| AI-02 | Phase 4 | Pending |
| AI-03 | Phase 4 | Pending |
| AI-04 | Phase 4 | Pending |
| AI-05 | Phase 4 | Pending |
| AI-06 | Phase 4 | Pending |
| AI-07 | Phase 4 | Pending |
| MENU-01 | Phase 4 | Pending |
| MENU-02 | Phase 4 | Pending |
| MENU-03 | Phase 4 | Pending |
| MENU-04 | Phase 4 | Pending |
| MENU-05 | Phase 4 | Pending |
| CAMP-01 | Phase 4 | Pending |
| CAMP-02 | Phase 4 | Pending |
| CAMP-03 | Phase 4 | Pending |
| CAMP-04 | Phase 4 | Pending |
| CAMP-05 | Phase 4 | Pending |
| SAVE-01 | Phase 4 | Pending |
| SAVE-02 | Phase 4 | Pending |
| SAVE-03 | Phase 4 | Pending |

**Coverage:**
- v1 requirements: 81 total
- Mapped to phases: 81
- Unmapped: 0

---
*Requirements defined: 2026-03-17*
*Last updated: 2026-03-17 after roadmap creation*
