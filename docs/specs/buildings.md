# Building System

## Phase 6-10: Building System

### Building System

| Address    | Name                          | Description                              |
|------------|-------------------------------|------------------------------------------|
| 0x0042e230 | Building_Init                 | Initializes building objects             |
| 0x0042e430 | Building_SetState             | Building state handler                   |
| 0x0042fd70 | Building_OnConstructionComplete | Called when building finishes           |
| 0x00433bb0 | Building_OnDestroy            | Called when building is destroyed        |
| 0x00432800 | Building_EjectPerson          | Ejects person from building              |
| 0x00434570 | Building_ApplyDamage          | Applies damage to building (offset 0x9E) |

## Appendix H: Building System Details

### Building States (offset 0x2C)

| State | Value | Description |
|-------|-------|-------------|
| Construction | 0x02 | Being built |
| Operating | 0x03 | Normal operation |
| Damaged | 0x04 | Taking damage |
| OnFire | 0x05 | Burning |
| Sinking | 0x06 | Sinking into water |

### Building Update (0x0042e5f0)

Per-tick building update handles:
1. Construction progress
2. Population spawning (housing)
3. Training queue (training buildings)
4. Damage/fire processing
5. Resource distribution

### Building Type Flags (DAT_005a0050)

Per building type, stride 0x4C (76 bytes):
| Bit | Description |
|-----|-------------|
| 0x01 | Can train units |
| 0x20 | Is housing |
| 0x40 | Is vehicle factory |
| 0x400 | Has special function |

---

## Appendix AH: Building Combat System

### Building_ProcessFightingPersons (0x00438610)

Manages combat between units inside buildings (guard towers, temples, etc.).

**Combat States:**

| State | Value | Description |
|-------|-------|-------------|
| 0 | Idle | Waiting for combat |
| 1 | Moving | Moving to position |
| 2 | Punch | Basic attack animation |
| 3 | Slash | Sword attack animation |
| 4 | Heavy | Heavy attack (super warrior) |
| 5 | Hit React | Being hit reaction |
| 6 | Block | Blocking attack |
| 7 | Knockback | Being knocked back |

**Combat Flow:**
1. Check if building has fighting persons
2. For each person in training state (0x19):
   - If sub-state 0: Move to combat position
   - If sub-state 1: Ready for combat
   - Random chance to attack nearby enemies
   - Process attack animations and damage

**Attack Selection:**
```c
// Random roll determines attack type
randomValue = Random() & 0xF;
if (unitType == SUPER_WARRIOR) {
    attackType = (randomValue > 6) ? HEAVY_ATTACK : PUNCH;
} else {
    if (randomValue < 5) attackType = PUNCH;
    else if (randomValue < 14) attackType = SLASH;
    else attackType = HEAVY_ATTACK;
}
```

**Damage Application:**
- Calls Combat_ProcessMeleeDamage for actual damage
- Updates victim state to HIT_REACT or BLOCK
- Rotates combatants to face each other

---

## Appendix BLD: Building System (Ghidra Disassembly Analysis)

### BLD.1 — Building Type Enum (Complete)

From GetObjectTypeName @ 0x00454050, building case at 0x454138, jump table at 0x454f58:

| Type | Value | Name | String Address | Category |
|------|-------|------|----------------|----------|
| 1 | 0x01 | Tepee | 0x5996DC | Housing |
| 2 | 0x02 | Tepee Stage 2 | 0x5996C0 | Housing |
| 3 | 0x03 | Tepee Stage 3 | 0x5996A4 | Housing |
| 4 | 0x04 | Drum Tower | 0x59968C | Defense |
| 5 | 0x05 | Temple | 0x599654 | Spell |
| 6 | 0x06 | Spy Train | 0x599640 | Training |
| 7 | 0x07 | Warrior Train | 0x599624 | Training |
| 8 | 0x08 | Super W Train | 0x599608 | Training |
| 9 | 0x09 | Reconversion Centre | 0x599664 | Training |
| 10 | 0x0A | Wall | 0x5995FC | Defense |
| 11 | 0x0B | Gate | 0x5995F0 | Defense |
| 12 | 0x0C | (type 12 / Ignore?) | 0x5995E0 | Unused? |
| 13 | 0x0D | BoatHut 1 | 0x5995CC | Vehicle |
| 14 | 0x0E | BoatHut 2 | 0x5995B8 | Vehicle |
| 15 | 0x0F | AirHut 1 | 0x5995A4 | Vehicle |
| 16 | 0x10 | AirHut 2 | 0x599590 | Vehicle |
| 17 | 0x11 | Guard Post | 0x599578 | Defense |
| 18 | 0x12 | Library | 0x599568 | Unused? |
| 19 | 0x13 | Prison | 0x599558 | Unused? |

Note: Wall (10), Gate (11), type 12, Library (18), Prison (19) appear unused in standard gameplay.
Buildable types with BLDG_MAX_BUILD_ config: TEEPEE1/2/3, DTOWER, TEMPLE, SPY, WARR, SWARR, BOAT, BALLOON.

### BLD.2 — Building Object Struct Layout

Building objects are allocated from the global object array at 0x878928 (index * 4 = pointer).
The struct is approximately 0xB0 bytes. All offsets from object base pointer (ESI in disassembly).

**Core identity fields:**
| Offset | Size | Field | Notes |
|--------|------|-------|-------|
| +0x0C | dword | flags | Bit flags (see BLD.3) |
| +0x0E | byte | flags2 | Bit 0x10 = ghost/preview mode |
| +0x10 | dword | flags3 | Bit 0x100000 = fighting_active, 0x200000 = fighting_timer |
| +0x14 | dword | flags4 | Bit 0x40 = wobble, 0x80 = invulnerable |
| +0x20 | word | next_link | Linked list next (object index) |
| +0x24 | word | object_id | Unique object ID |
| +0x26 | word | rotation | Angle 0-0x7FF (0 to 2*pi) |
| +0x2A | byte | object_type | 1=person, 2=building, etc. |
| +0x2B | byte | building_type | Building subtype (1-0x13, see BLD.1) |
| +0x2C | byte | state | State machine (see BLD.4) |
| +0x2D | byte | sub_state | Used in fighting: 0-7+ sub-states |
| +0x2E | byte | damage_flags | Bits 0-4 = damage types |
| +0x2F | byte | owner | Tribe index 0-3 |

**Position and terrain:**
| Offset | Size | Field | Notes |
|--------|------|-------|-------|
| +0x33 | word | tile_x | Position on map (tile X) |
| +0x35 | byte | terrain_flags | 0x08 = on water, 0x20 = needs redraw |
| +0x37 | word | anim_frame | Animation frame |
| +0x39 | byte | anim_sub | Animation sub-frame |
| +0x3A | byte | model_variant | Model variant |
| +0x3B | byte | terrain_type | Terrain type at location |
| +0x3D | word | world_x | World X position (16-bit) |
| +0x3F | word | world_z | World Z position (16-bit) |
| +0x41 | word | terrain_height | Terrain height at position |

**Building-specific fields:**
| Offset | Size | Field | Notes |
|--------|------|-------|-------|
| +0x57 | word | target_angle | Target rotation angle |
| +0x5D | word | current_facing | Current facing direction |
| +0x5F | word | action_timer | Generic action timer |
| +0x63 | word | wood_stored | Wood stored in building |
| +0x66 | word/ptr | specific_data | Building-specific data pointer |
| +0x68 | word | num_fighting | Number of occupants fighting |
| +0x6C | word | shake_x | Shake/wobble X offset |
| +0x6E | word | shake_z | Shake/wobble Z offset |
| +0x72 | word | target_person | Target person index (fighting) |
| +0x76 | byte | person_flags | Bit 0x4 = needs update |
| +0x78 | word | health_state | Health/HP state |
| +0x7A | word | base_x | Base position X (tile-aligned) |
| +0x7C | word | base_z | Base position Z (tile-aligned) |
| +0x7D | byte | saved_state | Previously saved state |
| +0x7E | dword | game_tick | Game tick when last updated |
| +0x82 | byte | misc_flags | Miscellaneous flags |
| +0x84 | word | linked_building | Linked building index |
| +0x85 | word | next_in_chain | Next person in chain (person index) |

**Occupant system:**
| Offset | Size | Field | Notes |
|--------|------|-------|-------|
| +0x86 | 6*word | occupant_slots | 6 occupant slots (person indices) |
| +0x92 | word | linked_person | Linked person index |
| +0x94 | word | shaman_person | Shaman/linked person (person index) |
| +0xA6 | byte | occupant_count | Current number of occupants |

**Training/conversion system:**
| Offset | Size | Field | Notes |
|--------|------|-------|-------|
| +0x96 | word | conversion_progress | Conversion progress counter |
| +0x98 | word | conversion_threshold | Conversion threshold value |
| +0x9A | word | flag_word | Status flags |
| +0x9C | word | status_flags | See BLD.3 for bit definitions |
| +0x9D | byte | more_flags | 0x04 = converted, 0x20 = ejected |
| +0xA0 | word | conversion_countdown | Active conversion timer |
| +0xA4 | word | training_countdown | Training completion timer |

**Damage/combat fields:**
| Offset | Size | Field | Notes |
|--------|------|-------|-------|
| +0x9E | word | damage_accumulated | Total damage taken |
| +0xA2 | word | attacking_person | Attacking person index |
| +0xA7 | byte | shake_duration | Shake timer (max 0x7F) |
| +0xAB | byte | damage_cooldown | Post-damage invulnerability timer |
| +0xAD | byte | wobble_duration | Wobble timer |
| +0xAE | byte | wobble_delay | Wobble delay between shakes |
| +0xAF | byte | last_attacker_tribe | Last attacker's tribe (0xFF = none) |

### BLD.3 — Building Flag Definitions

**Flags at +0x0C (dword):**
| Bit | Meaning |
|-----|---------|
| 0x00000001 | Dead/destroyed |
| 0x00000004 | On fire |
| 0x00000008 | Needs update |
| 0x00000010 | Moved |
| 0x00000040 | Active |
| 0x00100000 | Construction complete |
| 0x02000000 | Sinking |
| 0x08000000 | Needs footprint recalculation |
| 0x40000000 | Pending action |

**Status flags at +0x9C (word):**
| Bit | Meaning |
|-----|---------|
| 0x01 | Flag 1 |
| 0x02 | Wobbling |
| 0x04 | No occupants |
| 0x08 | Construction complete |
| 0x40 | Random timer active |
| 0x80 | Has room for more |
| 0x0400 | Flag 10 |

**Building type flags at 0x5A0050 (per-type, in type properties table):**
| Bit | Meaning |
|-----|---------|
| 0x01 | Convert type (training buildings) |
| 0x08 | Has occupant fighting |
| 0x20 | Spawn type (huts) |
| 0x40 | Vehicle type (boat/air huts) |
| 0x80 | Invulnerable flag |
| 0x0400 | Has conversion timer |

### BLD.4 — Building State Machine

States stored at offset +0x2C. Managed by Building_SetState @ 0x0042E430 (jump table).

```
State 1 (INIT)              → Construction start setup
State 2 (CONSTRUCTION_DONE) → Building_OnConstructionComplete called
State 3 (ACTIVE)            → Main operational state
State 4 (DESTROYING)        → Building_OnDestroy called
State 5 (SINKING)           → Sinking into ground
State 6 (FINAL)             → Final teardown/cleanup
```

Transitions:
- 1 → 2: Construction complete trigger
- 2 → 3: After OnConstructionComplete
- 3 → 4: Health depleted / destroy command
- 4 → 5: After destruction effects
- 5 → 6: After sinking complete

### BLD.5 — Building Type Properties Table

Located at 0x5A0014. Stride = type * 19 * 4 = type * 0x4C bytes.
Index computation in assembly: `type + (type + type*8)*2` = `type * 19`, then `*4` for byte offset.

**Table field offsets (from computed base):**
| Offset | Size | Field | Notes |
|--------|------|-------|-------|
| +0x14 | word | model_id | Animation/model ID |
| +0x28 | byte | max_occupants | Maximum occupants allowed |
| +0x39 | byte | conversion_target | Target person type for conversion |
| +0x46 | word | wood_cost | Wood cost to build |
| +0x50 | dword | behavior_flags | Controls active behavior (see BLD.3) |

### BLD.6 — Active Building Behavior Dispatch

When a building is in state 3 (ACTIVE), Building_Update @ 0x0042E5F0 reads flags from 0x5A0050
(within the type properties table) and dispatches to one of three handlers:

| Flag | Handler | Address | Description |
|------|---------|---------|-------------|
| 0x20 | Building_UpdateActive_TrainOrSpawn | 0x00430960 | Hut spawning / brave recruitment |
| 0x01 | Building_UpdateActive_Convert | 0x00430EF0 | Training building conversion pipeline |
| 0x40 | Building_UpdateActive_Vehicle | 0x00431970 | Boat/air hut vehicle production |

After the main behavior, these are always called:
- Building_UpdateWoodConsumption @ 0x00430430
- Building_UpdatePopGrowth @ 0x00430020
- Building_TriggerReconversion @ 0x00437860

### BLD.7 — Building Update Pipeline (Building_Update @ 0x0042E5F0)

Main tick function, ~0x370 bytes. Called every game tick for each active building.

```
1. Check building type 0x12 (special case)
2. Check flag 0x8000000 → recalc footprint (Building_UpdateFootprint) + occupancy (Building_RecalcOccupancy)
3. Damage cooldown timer at +0xAB (decrement each tick)
4. Fire damage check: flag 0x4 at +0x0C → Building_CheckFireDamage @ 0x434610
5. Wobble animation: flag 0x40 at +0x14 → shake X/Z offsets
6. Building_UpdateSmoke @ 0x434240
7. State-based dispatch (jump table on +0x2C minus 2, 5 cases):
   Case 0 (state 2 = ACTIVE):
     - Read type flags at 0x5A0050
     - Dispatch to TrainOrSpawn / Convert / Vehicle handler
     - Building_UpdateWoodConsumption
     - Building_UpdatePopGrowth
     - Building_TriggerReconversion
   Case 1 (state 3 = CONSTRUCTING):
     - Building_UpdateConstructing @ 0x4322B0
   Case 2 (state 4 = DESTROYING):
     - Building_UpdateDestroying @ 0x433E20
   Case 3 (state 5 = SINKING):
     - Building_UpdateSinking @ 0x4323D0
   Case 4 (state 6 = FINAL):
     - Teardown @ 0x4B3950
```

### BLD.8 — Building Init (Building_Init @ 0x0042E230)

Jump table on building_type (byte at +0x2B), 19 cases (1 to 0x12).
For each case, calls Building_InitFromType @ 0x0042E980 which:
- Sets initial position from parameters
- Sets orientation/rotation
- Sets initial state
- Reads type properties from table at 0x5A0014
- May spawn associated shaman (for Temple type)
- Some cases also call 0x436A50 (additional init)

At the end, stores terrain height at building position (+0x41).

### BLD.9 — Construction Complete (Building_OnConstructionComplete @ 0x0042FD70)

Large function (0x328 stack frame). Called when building transitions to state 2.

1. Sets +0x78 (health_state) to 4
2. Reads building type properties from 0x5A0014 table
3. Assigns shaman if one is present
4. Calculates terrain information at building position
5. Special handling for specific building types:
   - Type 4 (Drum Tower): special guard/defense setup
   - Type 0xD (BoatHut 1): vehicle production setup
   - Type 0xE (BoatHut 2): vehicle production setup
6. Calls Building_RecalcOccupancy @ 0x42ED70
7. Calls 0x436340 (footprint/terrain update)

### BLD.10 — Damage System (Building_ApplyDamage @ 0x00434570)

1. Checks global at 0x87E33F flag 0x4 — if set, god mode (no damage)
2. Checks invulnerability flag 0x80 at +0x14 — if set, skip damage
3. Adds damage amount to accumulated damage at +0x9E
4. Updates attacker tracking (person index at +0xA2, tribe at +0xAF)
5. Sets minimap dirty flag for redraw

### BLD.11 — Destruction System (Building_OnDestroy @ 0x00433BB0)

1. Spawns debris at 6 surrounding positions
   - Uses 3-byte entries from terrain data for debris placement
2. Checks for nearby buildings — may cause chain damage
3. Spawns destruction visual effects
4. Cleans up occupant references

### BLD.12 — Occupant System (Building_EjectPerson @ 0x00432800)

Buildings have 6 occupant slots at offset +0x86 (6 x word = person indices).
Occupant count at +0xA6.

**EjectPerson logic:**
1. Searches 6 slots for matching person index
2. Decrements occupant count at +0xA6
3. Positions ejected person at building location
4. Sets facing direction for ejected person
5. Handles torus world wrapping for position

**Related functions:**
- Building_HasRoomForOccupant @ 0x00434090: checks if occupant count < max
- Building_RecalcOccupancy @ 0x0042ED70: recounts occupants and updates state
- Building_CheckOccupantStatus @ 0x004348F0: checks occupant health/validity

### BLD.13 — Hut Spawning System (Building_UpdateActive_TrainOrSpawn @ 0x00430960)

For buildings with flag 0x20 (huts / Drum Tower):

1. Special handling for type 4 (Drum Tower) — guard/defense spawning
2. For regular huts: searches 9 surrounding tiles (3x3 grid) for valid braves to recruit
3. Braves must be:
   - Same tribe as building owner
   - Not already assigned to a building
   - In valid state for recruitment
4. Uses HUT_SPROG_TIME config values for spawn timing

### BLD.14 — Training System (Building_UpdateActive_Convert @ 0x00430EF0)

For buildings with flag 0x01 (training buildings — Spy/Warrior/Super Warrior/Reconversion):

1. Manages wood requirements for conversion
   - Reads wood cost from person type table at 0x59FBD2
   - Checks building wood storage at +0x63
2. Manages person conversion pipeline:
   - Person enters building (occupant slot)
   - Conversion timer counts down at +0xA0
   - Timer threshold from CONV_TIME_ config values
3. When timer expires:
   - Person type changes to target type (from +0x39 in type properties)
   - Person is ejected with new type
4. Occupant slot management for queuing

### BLD.15 — Vehicle Production (Building_UpdateActive_Vehicle @ 0x00431970)

For buildings with flag 0x40 (BoatHut/AirHut):
- Produces boats or airships
- Uses wood and person resources
- Vehicle spawns at building location

### BLD.16 — Building Fighting (Building_ProcessFightingPersons @ 0x00438610)

Very large function (~0xA00 bytes). Manages combat for persons inside buildings.

**Sub-states (at person +0x2D):**
- State 0: Idle/waiting
- State 1: Selecting target
- State 2: Moving to attack position
- State 3: Attacking
- State 4: Taking damage
- State 5: Retreating
- State 6: Dead/ejected
- State 7+: Special states

Uses PRNG at 0x885710 extensively for:
- Target selection randomization
- Damage calculation variance
- Attack timing jitter

### BLD.17 — AI Building Placement (AI_Cmd_BuildingPlacement @ 0x00445910)

7-state state machine for AI-controlled building placement:

| State | Description |
|-------|-------------|
| 0 | Init — validate building type and resources |
| 1 | Find placement location |
| 2 | Validate terrain and clearance |
| 3 | Check wood availability |
| 4 | Assign builder |
| 5 | Wait for construction start |
| 6 | Monitor construction |

### BLD.18 — AI Building Priorities (AI_ExecuteBuildingPriorities @ 0x0041B8D0)

10 priority slots, 12 building priorities. Uses bubble-sort to order priorities by urgency.

Priority types correspond to building needs:
- Housing (tepees needed for population)
- Defense (drum towers when under threat)
- Training (warrior/spy huts when mana available)
- Vehicle (boat/air huts for transport)
- Temple (spell access)

### BLD.19 — UI Building Info (UI_RenderBuildingInfo @ 0x004937F0)

Large UI rendering function (~0xA8E bytes). Displays when player selects a building:
- Building name (from GetObjectTypeName)
- Population count (current occupants / max)
- Wood stored
- Training progress bar (for training buildings)
- Conversion timer display

### BLD.20 — Configuration Constants

From constant.dat string references:

**Wood costs (WOOD_):**
- WOOD_HUT_1, WOOD_HUT_2, WOOD_HUT_3 — tepee construction costs
- WOOD_DRUM_TOWER — drum tower cost
- WOOD_TEMPLE — temple cost
- WOOD_SPY — spy training hut cost
- WOOD_WARRIOR — warrior training hut cost
- WOOD_SUPER — super warrior training hut cost
- WOOD_RECONV — reconversion centre cost
- WOOD_BOAT_1 — boat hut cost
- WOOD_AIR_1 — air hut cost
- WOOD_PREACH — preacher (reconversion) wood cost per conversion

**Conversion times (CONV_TIME_):**
- CONV_TIME_TEMPLE
- CONV_TIME_SPY
- CONV_TIME_WARRIOR
- CONV_TIME_SUPER
- CONV_TIME_RECONV

**Spawn times:**
- HUT_SPROG_TIME_1 (tepee 1)
- HUT_SPROG_TIME_2 (tepee 2)
- HUT_SPROG_TIME_3 (tepee 3)

**Building limits (BLDG_MAX_BUILD_):**
- BLDG_MAX_BUILD_TEEPEE1/2/3
- BLDG_MAX_BUILD_DTOWER
- BLDG_MAX_BUILD_TEMPLE
- BLDG_MAX_BUILD_SPY
- BLDG_MAX_BUILD_WARR
- BLDG_MAX_BUILD_SWARR
- BLDG_MAX_BUILD_BOAT
- BLDG_MAX_BUILD_BALLOON

**Building values (BLDG_V_):**
- BLDG_V_TEEPEE
- BLDG_V_FARM
- BLDG_V_DTOWER
- BLDG_V_TEMPLE
- BLDG_V_SWARR

**Population (MAX_POP_VALUE):**
- MAX_POP_VALUE — maximum population cap

### BLD.21 — Key Global Addresses

| Address | Description |
|---------|-------------|
| 0x5A0014 | Building type properties table base |
| 0x5A0050 | Building type behavior flags (within properties table) |
| 0x878928 | Object pointer array (index * 4 = ptr) |
| 0x885710 | PRNG state (LCG) |
| 0x884C88 | Current player tribe index |
| 0x87E459 | Global terrain data pointer |
| 0x87E33F | God mode / global cheat flags |
| 0x59FBD2 | Person type wood costs table |
| 0x59F8DB | Person type properties table (stride 0x32?) |
| 0x59FE5F | Person value table (stride 50) |
| 0x88897C | Minimap tile array |

### BLD.22 — Function Reference

| Address | Name | Size | Description |
|---------|------|------|-------------|
| 0x0042E230 | Building_Init | ~0x200 | Init jump table on building_type, 19 cases |
| 0x0042E430 | Building_SetState | ~0x1C0 | State machine, jump table on state, 6 cases |
| 0x0042E5F0 | Building_Update | ~0x370 | Main tick: damage, fire, wobble, state dispatch |
| 0x0042E980 | Building_InitFromType | ~0x400 | Sets position, orientation, reads type properties |
| 0x0042ED70 | Building_RecalcOccupancy | ~0x350 | Recounts occupants, updates state flags |
| 0x0042F0C0 | Building_UpdateFootprint | ~0xC80 | Recalculates building footprint on terrain |
| 0x0042FD70 | Building_OnConstructionComplete | ~0x328 | Construction done: health, shaman, type-specific |
| 0x00430020 | Building_UpdatePopGrowth | ~0x400 | Population growth in huts |
| 0x00430430 | Building_UpdateWoodConsumption | ~0x530 | Wood usage for active buildings |
| 0x00430960 | Building_UpdateActive_TrainOrSpawn | ~0x590 | Hut spawning, 9-tile brave recruitment |
| 0x00430EF0 | Building_UpdateActive_Convert | ~0xA80 | Training conversion pipeline |
| 0x00431970 | Building_UpdateActive_Vehicle | ~0x940 | Vehicle production logic |
| 0x004322B0 | Building_UpdateConstructing | ~0x120 | Construction progress tick |
| 0x004323D0 | Building_UpdateSinking | ~0x190 | Sinking animation tick |
| 0x00432800 | Building_EjectPerson | ~0x500 | Eject person from occupant slot |
| 0x00433BB0 | Building_OnDestroy | ~0x270 | Destruction: debris, chain damage, effects |
| 0x00433E20 | Building_UpdateDestroying | ~0x420 | Destruction sequence tick |
| 0x00434090 | Building_HasRoomForOccupant | ~0x1B0 | Check if building has free occupant slot |
| 0x00434240 | Building_UpdateSmoke | ~0x3D0 | Smoke particle effects |
| 0x004345F0 | Building_CheckOccupantStatus | ~0x300 | Validate occupant health/state |
| 0x00434570 | Building_ApplyDamage | ~0x80 | Damage application with god mode check |
| 0x00434610 | Building_CheckFireDamage | ~0x2E0 | Fire damage tick processing |
| 0x00437860 | Building_TriggerReconversion | ~0x200 | Trigger reconversion process |
| 0x00438610 | Building_ProcessFightingPersons | ~0xA00 | Building combat AI, 7+ sub-states |
| 0x00445910 | AI_Cmd_BuildingPlacement | ~0x600 | AI building placement state machine |
| 0x0041B8D0 | AI_ExecuteBuildingPriorities | ~0x500 | AI priority-based building execution |
| 0x004937F0 | UI_RenderBuildingInfo | ~0xA8E | Building info UI panel rendering |
| 0x00454050 | GetObjectTypeName | ~0xEB8 | Object type to display name mapping |

### BLD.23 — PRNG Algorithm

Used extensively in building fighting (and throughout the game).
Location: 0x885710 (global state dword).

```
Algorithm (from disassembly):
  val = val * val * 9 + val * 289 + val * 8 + val * 0x24DF
  val = ROR(val, 13)
```

This is a Linear Congruential Generator variant with rotation.

## Appendix BPLC — Building Placement in Level Loading Context

This appendix documents how buildings are placed during level loading, bridging
the level loading pipeline (Appendix LVL) with the building system (Appendix BLD).

### BPLC.1 — Overview

When a level loads, the DAT file contains up to 2000 unit records (55 bytes each).
Each record with model type 2 (Building) triggers a specialized creation path that:

1. Extracts rotation from the unit's angle field (angle >> 9)
2. Writes a 20-byte creation command to a command buffer
3. Calls Object_Create to allocate a game object
4. Dispatches to Building_Init via Object_InitByType
5. Calls Building_InitFromType to set up footprint, rotation, cell ownership
6. Flattens terrain under the building footprint

### BPLC.2 — Level_LoadAndCreateObjects (0x0040C330)

This is the main level file loader. It reads the entire DAT file using streaming
reads and creates all game objects. The building-specific path is at 0x40C88E.

**Building creation path (0x40C88E–0x40C8F3):**
```asm
0040c88e: CMP AL,0x2              ; model type == Building?
0040c890: JNZ 0x0040c8f5          ; skip if not
0040c892: MOV EAX,[EDI + 0x6]     ; unit angle (dword at offset +0x06 in unit record)
0040c895: MOV ECX,[0x0087a9db]    ; g_object_create_cmd_buf_ptr
0040c89b: CDQ
0040c89c: AND EDX,0x1ff
0040c8a2: ADD EAX,EDX
0040c8a4: SAR EAX,0x9             ; rotation = angle >> 9
0040c8a7: MOV [ECX],EAX           ; cmd[0x00] = rotation
0040c8b3: MOV [ECX + 0x4],EBX    ; cmd[0x04] = 0
0040c8bd: MOV [ECX + 0x8],0x2    ; cmd[0x08] = 2 (building creation mode)
0040c8ca: MOV [ECX + 0xc],0xffffffff ; cmd[0x0C] = -1 (no linked object)
0040c8d7: MOV [ECX + 0x10],EBX   ; cmd[0x10] = 0
0040c8da: ADD [0x0087a9db],0x14   ; advance buffer by 0x14 (20 bytes)
0040c8e1: MOV [0x0087a9d2],0x1    ; g_object_create_flag = 1
; then: push position, push tribe, push subtype, push model_type
; JMP → CALL Object_Create
```

**Creation command buffer structure (20 bytes per entry):**

| Offset | Size | Field | Building value |
|--------|------|-------|----------------|
| 0x00 | 4 | Rotation | angle >> 9 |
| 0x04 | 4 | Generic field | 0 |
| 0x08 | 4 | Creation mode | 2 (building) |
| 0x0C | 4 | Link ID | -1 (none) |
| 0x10 | 4 | Flags | 0 |

**Post-creation pipeline (after all 2000 units processed):**
- Calls Level_PostCreateUnit (0x40D420) per created object
- Calls FUN_0040DFC0 (secondary building spawner from General type 6 objects)
- Calls Object_ProcessTransports

### BPLC.3 — Object_Create (0x004AFC70)

Allocates a game object from the free lists and initializes base fields.

**Key logic:**
```asm
; Lookup model type properties at 0x59F610 (stride 3 per type)
004afc7d: LEA ESI,[EDX + EDX*0x2 + 0x59f610]  ; type_info = &type_table[model_type * 3]
004afc89: MOV CL,byte ptr [ESI + 0x2]          ; flags byte
004afc8c: TEST CL,0x1                           ; bit 0 = priority alloc
```

**Free list selection:**
- Two free lists: g_object_freelist_a (0x8788B4) and g_object_freelist_b (0x8788B8)
- Object count limit: current - used > 0x44C (1100) objects available
- Flag 0x02 in type_info: if used count > 0x250 (592), use freelist_a
- Priority objects (flag 0x01) or type 5/Scenery with special flags use freelist_b

**Object initialization:**
```asm
004afdb7: MOV ECX,0x2c            ; clear 0xB3 bytes (44 dwords + 1 word + 1 byte)
004afdbc: XOR EAX,EAX
004afdbe: STOSD.REP ES:EDI        ; zero-fill the object
; Then restore list pointers and ID
004afe17: MOV byte ptr [ESI + 0x2a],AL   ; obj+0x2A = model_type
004afe1e: MOV byte ptr [ESI + 0x2b],CL   ; obj+0x2B = subtype
004afe21: MOV byte ptr [ESI + 0x2f],DL   ; obj+0x2F = tribe_index
004afe24: LEA EDX,[ESI + 0x3d]           ; obj+0x3D = position (6 bytes)
004afe27: MOV ECX,dword ptr [EAX]        ; copy position from argument
004afe29: MOV dword ptr [EDX],ECX
004afe2d: MOV AX,word ptr [EAX + 0x4]
004afe31: MOV word ptr [EDX + 0x4],AX
```

**Command buffer consumption (building path):**
```asm
004afe5f: CMP byte ptr [0x0087a9d2],CL   ; g_object_create_flag set?
004afe65: JZ 0x004afe74                   ; skip if not
004afe67: OR dword ptr [ESI + 0xc],0x400  ; set flag 0x400 in obj+0x0C
004afe6e: MOV byte ptr [0x0087a9d2],CL   ; clear flag
004afe74: PUSH ESI
004afe75: CALL 0x004af950                 ; Object_InitByType
```

Flag 0x400 in obj+0x0C signals that a creation command is pending in the buffer.

### BPLC.4 — Object_InitByType (0x004AF950)

Jump table dispatch by model type (obj+0x2A), 11 entries:

| Model Type | Value | Init Function |
|------------|-------|---------------|
| Person | 1 | 0x4FD260 |
| **Building** | **2** | **0x42E230 (Building_Init)** |
| Creature | 3 | 0x483270 |
| Vehicle | 4 | 0x497A10 |
| Scenery | 5 | 0x4BCDE0 |
| General | 6 | 0x45FE00 |
| Effect | 7 | 0x4F0E20 |
| Shot | 8 | 0x4573E0 |
| Shape | 9 | 0x48F8D0 |
| Internal | 10 | 0x4ECF50 |
| Spell | 11 | 0x495440 |

After the type-specific init, sets flags:
```asm
004af9c0: OR dword ptr [ESI + 0x10],0x20000000  ; mark initialized
004af9c7: OR dword ptr [ESI + 0x14],0x4          ; mark active
```

### BPLC.5 — Object_SetStateByType (0x004AFA10)

Companion to Object_InitByType — sets runtime state per model type.

| Model Type | Value | State Function |
|------------|-------|----------------|
| Person | 1 | 0x4FD5D0 |
| **Building** | **2** | **0x42E430** |
| Creature | 3 | 0x483580 |
| Vehicle | 4 | 0x497BD0 |
| Scenery | 5 | 0x4BD100 |
| General | 6 | 0x4600C0 |
| Effect | 7 | 0x4F1950 |
| Shot | 8 | 0x4576F0 |
| Shape | 9 | 0x48F9B0 |
| Internal | 10 | 0x4ED340 |
| Spell | 11 | 0x4958B0 |

### BPLC.6 — Building_Init (0x0042E230)

Dispatches by building subtype (obj+0x2B, values 1-19). Jump table at 0x42E3E4.
All 19 subtypes call Building_InitFromType (0x42E980) as their core initialization.

**Special cases by subtype:**
- **Subtypes 13-14 (Guard Post, Library)**: Also call 0x436A50 with arg 0
- **Subtype 16 (Vault of Knowledge)**: Sets tribe to 0xFF (unowned), sets state to 2 (active), sets flag 0x100000 in obj+0x0C, calls 0x4B0AD0 with building size from type table
- **Subtype 18 (Prison)**: Sets state to 2 (active), clears flag 0x80 in obj+0x14, sets flag 0x100000

**Common epilogue (all subtypes):**
```asm
0042e3bb: MOV byte ptr [ESI + 0xaf],0xff   ; obj+0xAF = 0xFF
; Then calls 0x4E8E50 to compute terrain height at position
0042e3cc: CALL 0x004e8e50                   ; TerrainHeight(x, z)
0042e3d1: MOV word ptr [ESI + 0x41],AX      ; obj+0x41 = terrain height
0042e3d8: AND dword ptr [ESI + 0x10],0xfffffbff  ; clear flag 0x400
```

### BPLC.7 — Building_InitFromType (0x0042E980)

Core building initialization called from Building_Init. This is the critical
function that sets up position, rotation, footprint, and cell ownership.

**Step 1 — Snap to cell grid:**
```asm
0042e98e: CALL 0x004364e0           ; Building_SnapToGrid(obj)
; Copies position words and aligns to cell boundaries
0042e993: MOV AX,word ptr [ESI + 0x3f]
0042e99a: MOV word ptr [ESI + 0x7c],AX   ; obj+0x7C = aligned Z
0042e9a1: MOV CX,word ptr [EDI]           ; EDI = &obj+0x3D (position)
0042e9ae: MOV word ptr [ESI + 0x7a],CX   ; obj+0x7A = aligned X
; Both X and Z are masked with 0xFE00 (align to 512-unit grid)
```

**Step 2 — Link object to cell:**
```asm
0042e9bb: CALL 0x004b0840           ; Object_LinkToCell(obj, &obj+0x3D)
```

**Step 3 — Set building flag:**
```asm
0042e9c6: OR EAX,0x8000000          ; set flag 0x8000000 in obj+0x0C
0042e9d3: OR EAX,0x40               ; set flag 0x40 (building marker)
```

**Step 4 — Consume creation command buffer (if flag 0x400 set):**
```asm
0042e9d6: TEST AH,0x4              ; test flag 0x400 in obj+0x0C
0042e9dc: JZ 0x0042e9f3            ; skip if no command pending
0042e9de: AND EAX,0xfffffbff       ; clear flag 0x400
0042e9e6: SUB [0x0087a9db],0x14    ; rewind cmd buffer by 20 bytes
0042e9ed: MOV EBP,[0x0087a9db]     ; EBP = creation command ptr
; Read command fields:
0042e9fa: SHL AX,0x9               ; cmd[0] << 9 = angle
0042e9fe: MOV [ESI + 0x26],AX      ; obj+0x26 = angle
0042ea05: MOV [ESI + 0x82],CX      ; obj+0x82 = cmd[4] (generic)
0042ea0f: TEST EAX,EAX             ; cmd[0xC] (link ID)
0042ea11: JL 0x0042ea17            ; skip if -1
0042ea13: MOV [ESI + 0x63],AX      ; obj+0x63 = wood/link value
0042ea1b: MOV BL,[EBP + 0x8]       ; BL = cmd[8] = creation mode
```

**Step 5 — Set building state:**
```asm
; If creation mode (BL) != 0 and not from save (flag 0x10):
0042ea20: CALL Object_ClearStateByType_Stub  ; (no-op, just RET)
0042ea29: MOV byte ptr [ESI + 0x2c],BL      ; obj+0x2C = state from cmd[8]
0042ea2d: CALL Object_SetStateByType          ; set runtime state
; If no command buffer (EBP=0): default state = 2 (active)
0042ea7c: MOV byte ptr [ESI + 0x2c],0x2
```

**Step 6 — Update footprint and flatten:**
```asm
0042ea46: CALL 0x0042f0c0           ; Building_UpdateFootprint(obj)
0042ea51: CALL 0x0042ed70           ; Building_MarkFootprintCells(obj, 1)
0042ea5d: CMP byte ptr [ESI + 0x2b],0xa  ; subtype != 10 (Wall)?
0042ea60: CALL 0x0042f2a0           ; Building_FlattenTerrain(obj)
```

**Step 7 — Spawn linked General object (random chance):**
If the building type has a spawn chance (type_table[subtype]+0x46 != 0):
```asm
; PRNG check: ROR(seed, 13) & 0xF >= 10 → spawn
0042eb42: PUSH 0x9                  ; subtype 9 (Shape)
0042eb44: PUSH 0x6                  ; model type 6 (General)
0042eb46: CALL Object_Create        ; create linked scenery
0042eb58: MOV [EAX + 0x94],CX      ; link spawned object ↔ building
0042eb66: MOV [ESI + 0x94],AX
```

### BPLC.8 — Object_LinkToCell (0x004B0840)

Links a game object into the cell grid's linked list.

**Cell address computation:**
```asm
; From position (6 bytes: x_lo, x_hi, z_lo, z_hi, y_lo, y_hi):
004b0852: MOV BX,[EAX]              ; x word
004b0856: MOV AX,[EAX + 0x2]        ; z word
004b085a: MOV byte ptr [ESP + 0xa],BH  ; cell_x = x >> 8
004b0860: MOV byte ptr [ESP + 0xb],AH  ; cell_z = z >> 8
; cell_index = ((cell_x & 0xFE) * 2) | (cell_z & 0xFE00)
004b0885: LEA ESI,[EAX*0x4 + 0x88897c]  ; cell = &g_cell_grid[cell_index]
```

**Cell grid structure (at g_cell_grid = 0x88897C, stride 0x10 per cell):**

| Offset | Size | Field |
|--------|------|-------|
| +0x00 | 4 | Flags (terrain type, building flag 0x10, etc.) |
| +0x04 | 2 | Terrain height |
| +0x06 | 2 | Object linked list head (object ID) |
| +0x08 | 2 | Building ownership (object ID + flags) |
| +0x0A | 2 | Reserved |
| +0x0B | 1 | Owner nibble (low 4 bits) |
| +0x0C | 2 | Reserved |
| +0x0E | 1 | Altitude control nibble (low 4 bits) |

**Linked list insertion:**
```asm
004b088c: MOV AX,[ESI + 0x6]       ; old_head = cell.obj_list_head
004b0890: MOV [ECX + 0x20],AX      ; obj.next = old_head
; If old_head != 0: lookup obj and set obj.prev = new_obj
004b08a7: MOV [ESI + 0x6],DX       ; cell.obj_list_head = new_obj.id
004b08ac: OR [ECX + 0xc],0x20000   ; set "linked to cell" flag
```

Object lookup table at 0x878928: `object_ptr = [0x878928 + obj_id * 4]`

### BPLC.9 — Building_UpdateFootprint (0x0042F0C0)

Computes the building's corner position from its center + rotation + footprint shape.

**Rotation handling (4 orientations):**
```asm
; rotation = (obj+0x26 + 0x1FF) >> 9, gives 0-3
0042f0d3: CDQ
0042f0d4: AND EDX,0x1ff
0042f0dd: ADD EAX,EDX
0042f0e1: SAR EAX,0x9              ; rotation index 0-3
```

**Shape lookup:**
```asm
; shape_index = shape_data[rotation_table_offset + 0x2C]
0042f0f2: MOVSX EBP,byte ptr [ECX + EDX*0x1 + 0x2c]
0042f0f7: SHL EBP,0x4
0042f0fa: LEA EBP,[EBP + EBP*0x2]   ; shape_entry = index * 48
0042f0fe: ADD EBP,[0x005a7d78]       ; + g_building_footprint_table_ptr
```

**Footprint entry structure (48 = 0x30 bytes):**

| Offset | Size | Field |
|--------|------|-------|
| +0x00 | 1 | Width (in cells) |
| +0x01 | 1 | Height (in cells) |
| +0x02 | 1 | X origin offset |
| +0x03 | 1 | Z origin offset |
| +0x04-0x2B | 40 | Per-cell footprint mask (1 byte per cell) |
| +0x2C | 4 | Shape data reference |

**Corner calculation by rotation (jump table at 0x42F274):**

| Rotation | Corner X formula | Corner Z formula |
|----------|-----------------|-----------------|
| 0 | shape[+0x30] - shape[+0x20] | shape[+0x34] - shape[+0x24] |
| 1 | shape[+0x34] - shape[+0x24] | shape[+0x20] - shape[+0x30] + 0x200 |
| 2 | shape[+0x20] - shape[+0x30] + 0x200 | shape[+0x24] - shape[+0x34] + 0x200 |
| 3 | shape[+0x24] - shape[+0x34] + 0x200 | shape[+0x30] - shape[+0x20] |

Final position: `corner = aligned_pos + (offset << 8)`

### BPLC.10 — Building_MarkFootprintCells (0x0042ED70)

Iterates over the footprint grid and marks each cell as occupied by the building.

**Per-cell operations (when footprint mask byte has bit 0x01 set):**
```asm
; Compute cell address from footprint position
0042ee92: LEA ESI,[EAX*0x4 + 0x88897c]  ; cell ptr
; Set owner nibble (low 4 bits of cell+0x0B):
0042eea2: OR AL,byte ptr [ESP + 0x1c]    ; owner = tribe + 1
0042eea6: MOV byte ptr [ESI + 0xb],AL
; Set building ID in cell+0x08 ownership field:
0042eead: XOR CX,AX                      ; merge with existing
0042eeb0: AND CX,0x3ff                   ; mask to 10 bits (object ID)
0042eeb5: XOR CX,AX
0042eeb8: MOV word ptr [ESI + 0x8],CX
; Set "has building" flag 0x10 in cell flags:
0042eebc: MOV EAX,[ESI]
0042eebe: OR EAX,0x10
0042eec1: MOV [ESI],EAX
```

**Per-cell mode flags based on arg (EBX):**

| Mode | Cell flag action |
|------|-----------------|
| 0 | Clear 0x200 (construction marker) |
| 1 | Set 0x200 (completed building) |
| 4 | Clear 0x20000 |

**Altitude control (mode == 1, for completed buildings):**
```asm
0042eef0: CALL 0x004eb260           ; Terrain_GetAltitude(cell)
0042eef8: CMP EAX,0xf              ; clamp to max 15
0042eefd: MOV EAX,0xf
0042ef02: MOV CL,[ESI + 0xe]       ; cell+0x0E altitude nibble
0042ef05: AND CL,0xf0              ; clear low nibble
0042ef08: OR CL,AL                 ; set altitude value (0-15)
0042ef0a: MOV [ESI + 0xe],CL
```

**Post-iteration: flatten terrain at footprint center:**
```asm
0042ef63: CALL 0x00487870           ; Terrain_FlattenArea(center, radius)
; radius = max(width/2, height/2) + 1
```

### BPLC.11 — Building_ValidatePlacement (0x004B5990)

Checks whether a building can be placed at a given position. Called by AI
building placement system (not during level load — level load bypasses validation).

**Step 1 — Basic validity check:**
```asm
004b5999: CALL 0x00499eb0           ; Object_IsValidPosition(obj)
004b59a1: TEST AL,AL
004b59a3: JNZ 0x004b59ac           ; continue if valid
004b59a5: XOR EAX,EAX              ; return 0 (invalid)
```

**Step 2 — Check building type allows placement on current terrain:**
```asm
; building_type_data at 0x5A072D, indexed by subtype*23
004b59e1: TEST byte ptr [ECX + 0x5a072d],0x1  ; type flag bit 0
004b59e8: JZ 0x004b5a43            ; skip terrain check if not set
```

**Step 3 — Check cell terrain flags:**
```asm
; Look up cell at aligned position
004b5a09: MOV CL,[EAX*0x4 + 0x888988]  ; cell+0x0C byte
004b5a17: AND CL,0xf                    ; terrain type nibble
; Compute terrain type properties
004b5a23: TEST byte ptr [EDX*0x2 + 0x5a3038],0x3e  ; forbidden terrain?
004b5a2b: JZ 0x004b5a34            ; OK if not forbidden
004b5a2d: XOR EAX,EAX              ; return 0 (invalid)
```

**Step 4 — Check cell building flags:**
```asm
004b5a34: TEST dword ptr [EAX],0x206  ; cell flags: existing building (0x200)
                                       ; or other blockers (0x06)
004b5a3a: JZ 0x004b5a43            ; OK if clear
004b5a3c: XOR EAX,EAX              ; return 0 (blocked)
```

**Step 5 — Check 8 neighboring cells for tribe ownership conflicts:**
The function checks cells at offsets (+2,+2), (+2,+4), (-2,+2), (-2,0),
(-2,-2), (0,-2), (+2,-2), (+2,0) around the base position.

```asm
; Compute tribe ownership bitmask:
004b5a52: MOV CL,[EAX + 0xc22]     ; player+0xC22 = tribe ownership data
004b5a5e: ADD CL,0x4
004b5a61: SHL EDX,CL               ; EDX = 1 << (tribe + 4)
; For each neighbor cell:
004b5a7b: MOV CL,[EAX*0x4 + 0x88898b]  ; cell+0x0F ownership byte
004b5a82: TEST EDX,ECX             ; our tribe owns this cell?
004b5a84: JZ next_neighbor          ; if not, check next
004b5a86: MOV EAX,0x1              ; return 1 (valid - our territory)
```

Returns 1 if any of the 8 neighbor cells belongs to the placing tribe.
Returns 0 if none do (can't build outside your territory).

### BPLC.12 — Building_FlattenTerrain (0x0042F2A0)

Flattens the terrain under a completed building by computing the average height
of all cells in the footprint and setting each cell to that average.

**Height sampling:**
- Iterates over footprint grid (width × height cells)
- For cells at grid edges (x=0xFE or z=0xFE), wraps around the toroidal map
- Reads height from cell+0x04 (signed 16-bit) at 4 adjacent positions
- Accumulates sum and tracks minimum height

**Height decision:**
```asm
; After accumulating all heights:
0042f4f9: CDQ
0042f4fa: IDIV [ESP + 0x3c]        ; average = sum / count
; Check building type flag:
0042f510: MOV EDX,[EDX*0x4 + 0x5a0050]  ; type_table[subtype].flags
0042f517: TEST EDX,0x20000         ; "use minimum height" flag?
0042f51d: JNZ 0x0042f521           ; if set, keep average
0042f51f: MOV EAX,ECX              ; else use minimum height
```

**Terrain write modes:**
```asm
; mode flag from type_table[subtype].flags:
0042f533: TEST EDX,0x40000         ; "strict flatten" flag
0042f539: JZ 0x0042f543
0042f53b: MOV [ESP+0x34],0x5       ; mode = 5 (all cells)
; Default mode = 1 (only cells with footprint mask bit set)
```

**Per-cell height write:**
For each footprint cell, writes the computed height to cell+0x04 at 4 surrounding
positions (current cell and 3 neighbors), creating a smooth height transition.

**Guard Post/Library special case (subtypes 13-14):**
Additional rotation-dependent height adjustment for cells adjacent to the
building footprint (entrance/exit cells).

### BPLC.13 — Level_PostCreateUnit (0x0040D420)

Called after Object_Create for each unit during level load. Dispatches by
model type via jump table at 0x40D6D8.

**Building path (model type 2, at 0x40D4C7):**
```asm
; If not from save (flag 0x10 not set):
0040d4d5: MOV AL,[ESI + 0x2c]      ; current state
0040d4d8: PUSH ESI
0040d4d9: MOV [ESI + 0x7d],AL      ; save state to obj+0x7D
0040d4dc: CALL Object_ClearStateByType_Stub  ; (no-op)
0040d4e4: MOV byte ptr [ESI + 0x2c],0x8     ; set state = 8 (loading)
0040d4e9: CALL Object_SetStateByType
0040d4f1: MOV byte ptr [ESI + 0x2d],0x1     ; obj+0x2D = 1 (initialized)
0040d4f5: OR dword ptr [ESI + 0xc],0x40000000  ; set "level loaded" flag
```

**General type path (model type 6, subtype 2 = "building spawner"):**
Creates linked objects at building spawner positions. If subtype 2 and state == 1:
```asm
; Creates a General/10 object at the same position
0040d5fc: PUSH 0x6                  ; model = General
0040d5fe: CALL Object_Create
```

### BPLC.14 — Secondary Building Spawner (FUN_0040DFC0)

Called after all units are created during level load. Iterates through all
game objects and finds General type 6 objects with state 3, then spawns
buildings at linked positions.

### BPLC.15 — Key Global Addresses

| Address | Name | Description |
|---------|------|-------------|
| 0x0087A9D2 | g_object_create_flag | Set when cmd buffer has pending entry |
| 0x0087A9DB | g_object_create_cmd_buf_ptr | Pointer into creation command buffer |
| 0x0088897C | g_cell_grid | Cell grid base (128×128, stride 0x10) |
| 0x00878928 | g_object_lookup_table | Object ID → pointer mapping |
| 0x008788B4 | g_object_freelist_a | Primary free object list |
| 0x008788B8 | g_object_freelist_b | Secondary free object list |
| 0x008788BC | g_object_active_list | Active object linked list |
| 0x00884BE9 | g_object_total_count | Total allocated objects |
| 0x00884BF1 | g_object_used_count | Currently in-use objects |
| 0x00885710 | g_prng_seed | PRNG state for building spawns |
| 0x0087E459 | g_building_shape_data_ptr | Pointer to shape/rotation data |
| 0x005A7D78 | g_building_footprint_table_ptr | Pointer to footprint shape table |
| 0x005A0014 | g_building_type_table | Building type properties (stride 0x4C) |
| 0x005A0050 | g_building_type_flags | Building type flags (in type table) |
| 0x005A072D | g_building_terrain_flags | Terrain restriction flags per type |
| 0x005A3038 | g_terrain_type_properties | Terrain type property table |
| 0x0059F610 | g_model_type_info | Model type properties (stride 3) |
| 0x00957059 | g_model_instance_counters | Per-type instance ID counters |

### BPLC.16 — Game Object Structure (Building-relevant fields)

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| +0x00 | 4 | prev_ptr | Previous object in linked list |
| +0x04 | 4 | next_ptr | Next object in linked list |
| +0x0C | 4 | flags_a | Primary flags (0x40=building, 0x400=cmd pending, 0x8000000=building init, 0x20000=cell-linked) |
| +0x10 | 4 | flags_b | Secondary flags (0x20000000=initialized, 0x400=needs update) |
| +0x14 | 4 | flags_c | Tertiary flags (0x4=active, 0x80=can be attacked) |
| +0x18 | 4 | timestamp | Creation tick |
| +0x20 | 2 | cell_next | Next obj in cell list |
| +0x22 | 2 | cell_prev | Previous obj in cell list |
| +0x24 | 2 | object_id | Unique object ID |
| +0x26 | 2 | angle | Rotation angle |
| +0x28 | 2 | reserved | |
| +0x2A | 1 | model_type | Model type (1-11) |
| +0x2B | 1 | subtype | Building subtype (1-19) |
| +0x2C | 1 | state | Current state |
| +0x2D | 1 | init_flag | Set to 1 after init |
| +0x2E | 1 | type_instance_id | Instance counter for this type |
| +0x2F | 1 | tribe_index | Owner tribe (0-3, 0xFF=unowned) |
| +0x33 | 2 | shape_index | Shape/rotation table index |
| +0x3D | 6 | position | World position (x:2, z:2, y:2) |
| +0x41 | 2 | terrain_height | Ground height at position |
| +0x43 | 6 | velocity | Movement delta (x:2, z:2, y:2) |
| +0x63 | 2 | wood_amount | Wood stored (buildings) |
| +0x7A | 2 | aligned_x | Cell-aligned X position |
| +0x7C | 2 | aligned_z | Cell-aligned Z position |
| +0x7D | 1 | saved_state | Previous state (during loading) |
| +0x82 | 2 | cmd_field | From creation command buffer |
| +0x87 | 2 | random_seed | Per-object PRNG seed |
| +0x94 | 2 | linked_obj_id | ID of linked object |
| +0x9E | 2 | damage | Building damage value |
| +0xAF | 1 | building_marker | Set to 0xFF for buildings |

### BPLC.17 — Full Building Placement Pipeline (Level Load)

```
Level_LoadAndCreateObjects (0x40C330)
  │
  ├─ Read 2000 unit records (55 bytes each) from DAT file
  │
  ├─ For each unit with model_type == 2 (Building):
  │   │
  │   ├─ Extract angle, compute rotation = angle >> 9
  │   ├─ Write 20-byte creation command to buffer at [g_object_create_cmd_buf_ptr]
  │   ├─ Set g_object_create_flag = 1
  │   │
  │   └─ Object_Create (0x4AFC70)
  │       ├─ Allocate from free list
  │       ├─ Zero-fill object (0xB3 bytes)
  │       ├─ Set model_type, subtype, tribe, position
  │       ├─ Set flag 0x400 (command pending)
  │       │
  │       └─ Object_InitByType (0x4AF950)
  │           │
  │           └─ Building_Init (0x42E230)
  │               │
  │               └─ Building_InitFromType (0x42E980)
  │                   ├─ Building_SnapToGrid (0x4364E0)
  │                   ├─ Object_LinkToCell (0x4B0840)
  │                   ├─ Consume creation command → rotation, state
  │                   ├─ Building_UpdateFootprint (0x42F0C0)
  │                   │   └─ Compute corner from center + rotation + shape
  │                   ├─ Building_MarkFootprintCells (0x42ED70)
  │                   │   └─ Mark cells: owner, building flag, altitude
  │                   └─ Building_FlattenTerrain (0x42F2A0)
  │                       └─ Average/min height across footprint cells
  │
  ├─ Level_PostCreateUnit (0x40D420) per object
  │   └─ Building: save state, set state=8 (loading), set init flags
  │
  ├─ Secondary Building Spawner (0x40DFC0)
  │   └─ Find General type 6 state 3 objects → spawn buildings
  │
  └─ Object_ProcessTransports
```

### BPLC.18 — Comparison: Level Load vs Runtime Placement

| Aspect | Level Load | Runtime (AI/Player) |
|--------|-----------|-------------------|
| Validation | None (trusts DAT data) | Building_ValidatePlacement checks terrain, territory, existing buildings |
| Rotation | From unit angle >> 9 | Player chooses / AI selects |
| Command buffer | Written by Level_LoadAndCreateObjects | Written by placement UI / AI |
| Terrain flatten | Always performed | Always performed |
| State | Loaded as state 2 (active), then set to 8 (loading) | Starts at state 0 (init/construction) |
| Territory check | Not performed | Must be in tribe's territory (8-neighbor check) |

### BPLC.19 — Function Reference

| Address | Name | Size | Description |
|---------|------|------|-------------|
| 0x0040C330 | Level_LoadAndCreateObjects | ~0x810 | Main level loader, creates all objects |
| 0x0040D420 | Level_PostCreateUnit | ~0x2B8 | Post-creation init per unit type |
| 0x0040DFC0 | (secondary spawner) | ~0x190 | Spawn buildings from General objects |
| 0x0042E230 | Building_Init | ~0x1B1 | Building subtype dispatch |
| 0x0042E980 | Building_InitFromType | ~0x203 | Core building initialization |
| 0x0042ED70 | Building_MarkFootprintCells | ~0x203 | Mark cells as building-occupied |
| 0x0042F0C0 | Building_UpdateFootprint | ~0x1B3 | Compute corner from rotation + shape |
| 0x0042F2A0 | Building_FlattenTerrain | ~0x4DD | Flatten terrain under building |
| 0x004AF950 | Object_InitByType | ~0x85 | Dispatch to type-specific init |
| 0x004AFA10 | Object_SetStateByType | ~0x84 | Dispatch to type-specific state set |
| 0x004AFAC0 | Object_ClearStateByType_Stub | 1 | No-op (single RET) |
| 0x004AFC70 | Object_Create | ~0x237 | Allocate object from free list |
| 0x004B0840 | Object_LinkToCell | ~0x78 | Link object into cell grid |
| 0x004B0950 | Object_MoveToPosition | ~0x17A | Move object to new cell position |
| 0x004B5990 | Building_ValidatePlacement | ~0x28E | Check if placement is legal |
| 0x004E8E50 | Terrain_InterpolateHeight | ~0xFF | Bilinear height interpolation |
| 0x0042E430 | Building_SetState | ~0x1A4 | 6-state building state machine |
| 0x004364E0 | Building_InitModelSelector | ~0xBA | Select the OBJS model from type data and PRNG |
| 0x0040DFC0 | Level_SpawnBuildingsFromGenerals | ~0x192 | Post-load building spawner |
| 0x00436340 | Building_ResetFireEffects | ~0x15F | Reset fire effects on footprint |
| 0x004B0AD0 | Object_SetShapeFromType | ~0x6B | Set shape from type properties table |
| 0x0049BBA0 | Shape_LoadDatFile | ~0x5C | Load SHAPES.DAT footprint data |
| 0x0049B9B0 | Shape_LoadBankData | ~0x1E9 | Load shape bank + patch pointers |
| 0x0049BC40 | Shape_PatchPointers | ~0xC2 | Fix up shape data internal pointers |

### BPLC.20 — SHAPES.DAT File Format

The building footprint data is stored in `objects/SHAPES.DAT` (4604 bytes).

**File structure:**
- 95 entries × 48 (0x30) bytes each = 4560 bytes + possible header/padding
- Entry 0 is null (all zeros)
- Entries are grouped in sets of 4 for the 4 rotation orientations

**Footprint entry (48 bytes):**

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| +0x00 | 1 | width | Width in cells |
| +0x01 | 1 | height | Height in cells |
| +0x02 | 1 | origin_x | X origin offset (cells) |
| +0x03 | 1 | origin_z | Z origin offset (cells) |
| +0x04 | 40 | mask[w×h] | Per-cell flags (bit 0x01 = occupied) |
| +0x2C | 4 | shape_ref | Offset into shape data (patched at load) |

**Per-cell mask bits:**

| Bit | Mask | Description |
|-----|------|-------------|
| 0 | 0x01 | Cell is part of footprint |
| 1-2 | 0x06 | Cell type flags |
| 3 | 0x08 | Gate/entrance cell (Guard Post, Library) |
| 4-7 | 0xF0 | Additional flags |

**Footprint sizes found in SHAPES.DAT:**

| Size | Cell count | Building types |
|------|-----------|----------------|
| 3×3 | 9 | Small Hut (subtypes 1-3) |
| 4×4 | 16 | Drum Tower (4), some special buildings |
| 5×5 | 25 | Temple (5), Training huts (6-8), Guard Post (13) |
| 4×5 / 5×4 | 20 | Boat Hut (11-12, rotated) |
| 5×6 / 6×5 | 30 | Air Hut (14-15, rotated) |
| 6×6 | 36 | Large buildings |
| 7×7 | 49 | Vault of Knowledge (19) |

**Rotation groups (4 consecutive entries per group):**
- Group 0 (entries 1-4): 3×3 — Small Hut rotation 0-3
- Group 1 (entries 5-8): 4×4 — Drum Tower rotation 0-3
- Group 2 (entries 9-12): 5×5 — Temple rotation 0-3
- Group 3 (entries 13-16): 4×4 + 5×5 — mixed training
- Group 12 (entries 49-52): 7×7 — Vault of Knowledge

### BPLC.21 — Building_InitModelSelector (0x004364E0)

Sets the initial OBJS model selector in obj+0x63 based on building type properties.

**Logic:**
- Reads type flags at `type_table[subtype*0x4C + 0x5A0008 + 0x49]` (byte)
- If flag 0x20 is set (hut/spawning building): uses PRNG to select one of three hut mesh families
  ```asm
  ; PRNG mod 3 selects family:
  ;   family 0: subtype_offset + 0x6B (107)
  ;   family 1: subtype_offset + 0x77 (119)
  ;   family 2: subtype_offset + 0x83 (131)
  ; Then adds tribe_index * 3. For hut subtypes 1-3,
  ; subtype_offset is subtype - 1.
  ```
- The native model-table pointer is based 38 records into the raw OBJS0
  archive, so native selectors 107/119/131 correspond to archive indices
  145/157/169 in extraction tools.
- If flag 0x20 is not set: uses a fixed model selector from `type_table[subtype*0x4C + 0x5A0008]` (word)
- If flag 0x40 is set: adds tribe_index to the fixed model selector

### BPLC.22 — Building_SetState (0x0042E430)

6-state building state machine. Jump table at 0x42E5D4.

| State | Value | Handler | Description |
|-------|-------|---------|-------------|
| Init/Construction | 1 | 0x42E45C | Set shape size, clear linked objects, call 0x4C3890 |
| Active | 2 | 0x42E4FA | Call Building_OnConstructionComplete (0x42FD70) |
| Destroying | 3 | 0x42E505 | Set flag 0x100000, set obj+0x67 = 2 |
| Sinking | 4 | 0x42E512 | Call Building_OnDestroy (0x433BB0) |
| Final/Cleanup | 5 | 0x42E51D | Set flags 0x2000000 + 0x100000 |
| Placement | 6 | 0x42E532 | Call 0x4B3920 (occupant evacuation), set shape from type |

**Common epilogue (all states):**
- If state != 2 and obj+0x84 != 0: destroy linked object via 0x4B1550
- If obj+0x92 != 0 and (state != 2 or not local player): destroy linked object

### BPLC.23 — Level_SpawnBuildingsFromGenerals (0x0040DFC0)

Iterates all game objects (stride 0xB3) from g_object_list_start (0x878910) to
g_object_list_end (0x87891C). For each General/6 subtype 6 with state 3:

**Search for linked building spawner:**
- Scans 10 linked object slots at obj+0x72 (2 bytes each)
- Looking for General/2 objects with obj+0x80 == 1
- If found, searches cell grid for a Scenery/9 (tree/stone marker)

**Building creation:**
```asm
; Gets rotation from scenery object's angle (>>9)
; Writes creation command: rotation, mode=2, link=-1
; Creates Building type 0x12 (subtype 18 = Prison) with tribe 0xFF
0040e10c: PUSH 0x12               ; subtype = Prison
0040e10e: PUSH 0x2                ; model = Building
; After creation: sets scenery state to 0xC, destroys it
```

### BPLC.24 — Terrain_InterpolateHeight (0x004E8E50)

Bilinear interpolation of terrain height at a sub-cell position.

**Input:** x_pos (word), z_pos (word) — full-precision world coordinates

**Process:**
1. Extract cell coordinates: cell_x = (x >> 1) & 0xFF, cell_z = (z >> 1) & 0xFF
2. Extract sub-cell fraction: frac_x = x & 0x1FE >> 1, frac_z = z & 0x1FE >> 1
3. Read 4 corner heights from cell grid (+0x04 in each cell):
   - h00 = cell[x,z]+0x04
   - h10 = cell[x+1,z]+0x04
   - h01 = cell[x,z+1]+0x04
   - h11 = cell[x+1,z+1]+0x04
4. Handle toroidal wrap: if cell_x or cell_z at 0xFE boundary, wraps around
5. Two interpolation paths based on cell flag bit 0x01:
   - **Flag clear (normal):** Standard bilinear interpolation
   - **Flag set (diagonal split):** Different triangle selection based on frac_x + frac_z < 256

**Output:** Interpolated height (word)

### BPLC.25 — Shape Data Structure (at g_building_shape_data_ptr = 0x87E459)

The shape data is a separate table from the footprint data (SHAPES.DAT).
Stride is 0x36 (54 bytes) per entry.

**Entry structure (54 bytes):**

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| +0x00 | 1 | flags | Bit 0x40 = special shape |
| +0x07 | 1 | shape_clear | Set to 0 during bank load |
| +0x10 | 4 | point_ptr_a | Pointer to point data (patched) |
| +0x14 | 4 | point_ptr_b | Pointer to point data (patched) |
| +0x18 | 4 | face_ptr_a | Pointer to face data (patched) |
| +0x1C | 4 | face_ptr_b | Pointer to face data (patched) |
| +0x2C | 1 | footprint_idx | Index into SHAPES.DAT footprint table |

**Pointer patching (Shape_PatchPointers, 0x49BC40):**
- Point pointers: `ptr = original_index * 0x3C + g_point_data_base - 0x3C`
- Face pointers: `ptr = original_index * 6 + g_face_data_base - 6`

### BPLC.26 — Updated Function Reference (Iteration 2)

Additional functions discovered in iteration 2:

| Address | Name | Size | Description |
|---------|------|------|-------------|
| 0x004364E0 | Building_InitModelSelector | ~0xBA | Select the OBJS model from type data and PRNG |
| 0x0042E430 | Building_SetState | ~0x1A4 | 6-state building state machine |
| 0x0040DFC0 | Level_SpawnBuildingsFromGenerals | ~0x192 | Post-load building spawner (Prison from General/6) |
| 0x004E8E50 | Terrain_InterpolateHeight | ~0xFF | Bilinear height interpolation with toroidal wrap |
| 0x004B0AD0 | Object_SetShapeFromType | ~0x6B | Set shape from type properties table |
| 0x00436340 | Building_ResetFireEffects | ~0x15F | Reset fire effects on footprint cells |
| 0x0049BBA0 | Shape_LoadDatFile | ~0x5C | Load SHAPES.DAT footprint data |
| 0x0049B9B0 | Shape_LoadBankData | ~0x1E9 | Load shape bank + patch pointers |
| 0x0049BC40 | Shape_PatchPointers | ~0xC2 | Fix up shape data internal pointers |

Data:

| Address | Name | Description |
|---------|------|-------------|
| 0x00598170 | g_shape_count | Number of shape entries loaded |

### BPLC.27 — Building_OnConstructionComplete (0x0042FD70)

Called when a building transitions from construction (state 1) to active (state 2)
via Building_SetState. This is the final step that makes a building fully operational.

**Stack frame:** 0x328 bytes local + 4 saved regs

**Key operations:**

1. **Set completion phase:**
   ```asm
   0042fd89: MOV byte ptr [ESI + 0x78],0x4    ; obj+0x78 = 4 (completion phase)
   0042fd9c: OR byte ptr [ESI + 0x9c],0x8     ; set flag 0x08 in obj+0x9C
   ```

2. **Set final shape from type properties:**
   ```asm
   0042fd8f: MOV CL,byte ptr [ESI + 0x2b]     ; CL = building subtype
   ; subtype * 0x13 (19) computed as: subtype + subtype*8*2
   0042fda3: MOV AX,word ptr [ECX*0x4 + 0x5a0014]  ; shape from type_table[subtype*0x4C + 0x14]
   0042fdad: CALL Object_SetShapeFromType      ; 0x4B0AD0
   ```

3. **Handle linked scenery object (obj+0x94):**
   ```asm
   0042fdb2: MOV AX,word ptr [ESI + 0x94]     ; linked object ID
   0042fdbf: JZ skip_linked                    ; skip if zero
   0042fdc4: MOV EAX,dword ptr [EAX*0x4 + 0x878928]  ; resolve object pointer
   0042fdcb: TEST byte ptr [EAX + 0xc],0x1    ; check dead flag
   0042fdd1: CMP byte ptr [EAX + 0x2a],BL     ; check model_type != 0
   0042fdd6: MOV EBX,EAX                      ; EBX = linked object
   ```

4. **Set linked object's shape (if linked object exists):**
   ```asm
   0042fde1: MOV byte ptr [EBX + 0x78],0x4    ; linked obj phase = 4
   0042fde5: MOV AL,byte ptr [EBX + 0x2d]     ; linked obj state
   ; Computes table index: state * 12
   0042fdee: MOV ECX,dword ptr [EAX + 0x5a05f8]  ; shape params from secondary table
   0042fdfd: CALL Object_SetShapeFromType
   ```

5. **Compute entrance position for Drum Tower (4), Guard Post (13), Library (14):**
   ```asm
   0042feb4: XOR EAX,EAX
   0042feb6: MOV AL,byte ptr [ESI + 0x2b]     ; subtype
   0042feb9: CMP EAX,0x4                      ; == Drum Tower?
   0042febc: JZ handle_entrance
   0042fec2: CMP EAX,0xd                      ; >= Guard Post (13)?
   0042fec5: JL skip_entrance
   0042fecb: CMP EAX,0xe                      ; <= Library (14)?
   0042fece: JG skip_entrance
   ```

   Entrance computation uses footprint data to find gate/entrance cell:
   ```asm
   ; Gets footprint_idx from shape_data[shape_index + rotation + 0x2C]
   0042fe4a: MOVSX EDX,byte ptr [ECX + EBX*0x1 + 0x2c]
   0042fe4f: SHL EDX,0x4                      ; footprint_idx * 48
   0042fe55: LEA EDX,[EDX + EDX*0x2]
   0042fe58: ADD EDX,dword ptr [0x005a7d78]   ; + footprint table base
   ; Read entrance offsets from footprint +0x06, +0x07
   0042fe67: MOVSX AX,byte ptr [EDX + 0x6]
   0042fe8e: SUB AX,CX                        ; subtract origin
   0042fe91: SHL AX,0x6                        ; scale to world coords
   0042fe95: ADD AX,word ptr [ESI + 0x7c]     ; add building corner Z
   ```
   Then calls 0x4A4E40 (entrance position setup) and for Guard Post/Library:
   - Calls 0x42FFF0 (compute entrance facing)
   - Calls 0x491770 (find gate cell in footprint)
   - Calls 0x436BE0 (validate entrance) — if invalid, spawns Effect/0x53 object

6. **Call Building_ResetFireEffects and Building_MarkFootprintCells:**
   ```asm
   0042ff7a: PUSH ESI
   0042ff7b: CALL Building_ResetFireEffects    ; 0x436340
   0042ff83: PUSH 0x1
   0042ff85: PUSH ESI
   0042ff86: CALL Building_MarkFootprintCells  ; 0x42ED70 — mark cells with building flags
   ```

7. **Compute spawn/vehicle timer based on type flags:**
   ```asm
   0042ffa4: MOV EAX,dword ptr [EDX*0x4 + 0x5a0050]  ; type_table[subtype*0x4C + 0x50]
   0042ffab: TEST AH,0x4                      ; test flag 0x400 (training building)
   0042ffae: JZ check_vehicle
   ; Training building: compute spawn timer
   0042ffb0: PUSH ECX                          ; subtype
   0042ffb4: PUSH EAX                          ; tribe
   0042ffb5: CALL 0x00426220                   ; get training time
   0042ffba: SUB AX,0x36                       ; subtract 54
   0042ffc1: MOV word ptr [ESI + 0xa4],AX     ; obj+0xA4 = spawn timer
   ;
   0042ffd3: TEST AL,0x40                      ; test flag 0x40 (vehicle building)
   0042ffd5: JZ done
   0042ffd7: MOV word ptr [ESI + 0xa4],0x0    ; vehicle timer = 0
   ```

**Footprint entry offsets +0x06 and +0x07:**
These are the entrance/gate position offsets (x, z) relative to the footprint origin.
Only used by Drum Tower, Guard Post, and Library — buildings with entrances.

### BPLC.28 — Object_IsValidPosition (0x00499EB0)

Validates whether an object can exist at its current cell position. Used during
runtime placement (not during level load). Returns AL=1 if valid, AL=0 if invalid.

**Two code paths based on subtype flag at 0x5A072D:**

Path 1 — Normal buildings (flag bit 0x01 at `[subtype*23 + 0x5A072D]` is clear):
```asm
00499f0c: TEST byte ptr [ECX + 0x5a072d],0x1  ; subtype property flag
00499f16: JNZ path2                            ; if set, use path 2

; Terrain type check: cell+0x0C low nibble → terrain properties
00499f18: AND CL,0xf                           ; terrain type = cell[+0x0C] & 0xF
00499f26: TEST byte ptr [EDX*0x2 + 0x5a3038],0x3c  ; terrain_props & 0x3C
00499f2e: JZ fail                              ; if zero = forbidden terrain

; Cell flags check
00499f30: TEST dword ptr [EDI],0x100004        ; cell[0] & 0x100004
00499f36: JNZ fail                             ; if any set = blocked

; Height limit check
00499f3c: MOV DL,byte ptr [ESI + 0x30]        ; obj+0x30 = height limit index
00499f42: MOV SI,word ptr [ESI + 0x5f]        ; obj+0x5F = current height
; Computes table index: idx*51 (5*5*2 + idx)
00499f4b: CMP word ptr [EDX + EDI*0x1 + 0x5a0974],SI  ; compare with max height
00499f53: JG fail                              ; if table value > current = invalid
```

Path 2 — Special buildings (flag bit 0x01 set):
```asm
; Terrain type check with stricter mask
00499f6b: TEST byte ptr [EDX*0x2 + 0x5a3038],0x3d  ; terrain_props & 0x3D (includes bit 0)
00499f73: JZ fail

; Cell flags check with stricter mask
00499f75: TEST dword ptr [EDI],0x100204        ; cell[0] & 0x100204 (adds 0x200)
00499f7b: JNZ fail

; Same height limit check as path 1
```

**Key differences between paths:**
- Path 1 (normal): terrain mask 0x3C, cell block mask 0x100004
- Path 2 (special): terrain mask 0x3D, cell block mask 0x100204 (also blocks on 0x200 = completed building)

**Cell address computation (reused across all cell-accessing functions):**
```asm
; From obj+0x3D (x position) and obj+0x3F (z position):
00499ec8: MOV CX,word ptr [ESI + 0x3d]        ; world X
00499ecc: MOV DX,word ptr [ESI + 0x3f]        ; world Z
00499ed0: MOV byte ptr [ESP + 0xa],CH          ; cell_x = X >> 8
00499ed6: MOV byte ptr [ESP + 0xb],DH          ; cell_z = Z >> 8
; cell_x &= 0xFE, cell_z &= 0xFE (even alignment)
00499ee6: AND ECX,0xfe
00499eee: AND EDX,0xfe00
00499ef4: OR ECX,EDX
00499ef6: LEA EDI,[ECX*0x4 + 0x88897c]        ; cell_ptr = grid_base + index*4
```

### BPLC.29 — Cell_GetBuildingAltitude (0x004EB260)

Computes the altitude control value for a cell based on the building occupying it.
Returns EAX = altitude (0-15, clamped).

**Input:** cell pointer (ESP+0x8)

**Logic:**

1. **Check if cell has a building:**
   ```asm
   004eb267: MOV CX,word ptr [ESI + 0x8]      ; cell+0x08 = building reference
   004eb26b: AND CX,0x3ff                      ; mask to 10-bit object ID
   004eb270: JZ no_building                    ; if zero = no building
   004eb272: TEST byte ptr [ESI + 0x1],0x2     ; cell+0x01 bit 0x02
   004eb276: JZ no_building                    ; must have building flag
   ```

2. **Resolve building object:**
   ```asm
   004eb282: MOV ECX,dword ptr [ECX*0x4 + 0x878928]  ; object_table[id]
   004eb289: TEST byte ptr [ECX + 0xc],0x1    ; dead flag
   004eb28f: CMP byte ptr [ECX + 0x2a],AL     ; model_type != 0
   004eb294: MOV EDX,ECX                      ; EDX = building object
   ```

3. **Two altitude calculation modes based on building state:**

   **Mode A — Construction in progress (obj+0x2C == 1):**
   ```asm
   004eb29a: CMP byte ptr [EDX + 0x2c],0x1    ; state == Init/Construction?
   004eb2a0: MOV CL,byte ptr [EDX + 0x2b]     ; building subtype
   ; Check type_table[subtype*0x4C + 0x51] bit 0x01
   004eb2b4: TEST byte ptr [EAX + 0x5a0051],0x1
   004eb2bb: JNZ mode_b                       ; if set, use mode B instead
   ; altitude = type_table[subtype*0x4C + 0x3D] * obj+0x78 / 3
   004eb2bd: MOVSX EAX,byte ptr [EAX + 0x5a003d]  ; base altitude from type table
   004eb2c4: MOVSX ECX,byte ptr [EDX + 0x78]  ; construction phase (0-4)
   004eb2c8: IMUL EAX,ECX                     ; altitude * phase
   004eb2cb: CDQ
   004eb2cc: MOV ECX,0x3
   004eb2d1: IDIV ECX                          ; / 3
   ```

   **Mode B — Completed building (or special type):**
   ```asm
   ; altitude = type_table[subtype*0x4C + 0x3D] (directly)
   004eb2e0: MOVSX EAX,byte ptr [ECX*0x4 + 0x5a003d]
   ```

4. **Walk linked object chain adding scenery altitude:**
   ```asm
   ; Iterate cell's object linked list via obj+0x06
   004eb2e8: MOVSX ECX,word ptr [ESI + 0x6]   ; next object in cell
   004eb2ec: MOV ESI,dword ptr [ECX*0x4 + 0x878928]
   004eb2f5: JZ done
   004eb2f7: MOV EDX,0x5                      ; check for Scenery type (5)
   004eb2fc: CMP EAX,0xf                      ; if altitude >= 15, cap and exit
   004eb301: CMP byte ptr [ESI + 0x2a],DL     ; model_type == Scenery?
   004eb308: MOV CL,byte ptr [ESI + 0x2b]     ; scenery subtype
   ; Add scenery altitude contribution from table at 0x5A07A3
   004eb30e: MOVSX ECX,byte ptr [ECX*0x8 + 0x5a07a3]
   004eb316: ADD EAX,ECX
   ```

5. **Clamp to 15:**
   ```asm
   004eb329: CMP EAX,0xf
   004eb32c: JLE done
   004eb32e: MOV EAX,0xf                      ; cap at 15
   ```

**Building type altitude table:** at `type_table[subtype*0x4C + 0x3D]` (signed byte)
**Scenery altitude table:** at `0x5A07A3 + scenery_subtype * 24` (signed byte)

### BPLC.30 — Building_OnDestroy (0x00433BB0)

Called when a building transitions to state 4 (Sinking) via Building_SetState.
Spawns rubble objects at footprint-adjacent positions and causes nearby persons to flee.

**Key operations:**

1. **Setup and clear flags:**
   ```asm
   00433bba: AND word ptr [ESI + 0x9c],0xfffd ; clear flag 0x02 in obj+0x9C
   00433bc7: OR byte ptr [ESI + 0x35],0x20     ; set flag 0x20 in obj+0x35
   00433bd3: MOV word ptr [ESI + 0x6c],BP      ; clear obj+0x6C (0)
   00433bda: MOV byte ptr [ESI + 0xa7],0x7f    ; obj+0xA7 = 0x7F (127)
   00433be5: MOV word ptr [ESI + 0x6e],BP      ; clear obj+0x6E (0)
   ```

2. **Get footprint data and compute corner position:**
   ```asm
   ; shape_index from obj+0x33, rotation from angle >> 9
   00433bf2: MOV EDX,dword ptr [0x0087e459]    ; shape data table
   00433c00: MOVSX EDI,byte ptr [ECX + EDX*0x1 + 0x2c]  ; footprint_idx
   00433c05: SHL EDI,0x4                       ; * 48
   00433c0c: LEA EDI,[EDI + EDI*0x2]
   00433c0f: ADD EDI,dword ptr [0x005a7d78]    ; footprint entry base
   ; Corner = building pos (obj+0x7A/0x7C) - origin * 256
   00433c15: MOVZX CX,byte ptr [EDI + 0x2]    ; origin_x
   00433c1a: SHL CX,0x8
   00433c1e: SUB AX,CX                        ; corner_x = pos_x - origin_x * 256
   ```

3. **Loop 6 adjacent positions from footprint+0x1A (3 bytes each):**
   ```asm
   ; EDI points to footprint_entry + 0x1A (adjacent position table)
   00433c21: ADD EDI,0x1a
   ; Loop counter: EBP from 0 to 5 (6 iterations)
   00433e04: CMP EBP,0x6
   00433e07: JL loop_start
   ```

   For each position (3 bytes: x_off, subtype, z_off):
   ```asm
   00433c42: MOV AL,byte ptr [EDI]             ; x_offset
   00433c44: MOV BL,byte ptr [EDI + 0x1]       ; rubble subtype index
   00433c4e: MOV CL,byte ptr [EDI + 0x2]       ; z_offset (if all 3 are zero, skip)
   ; Compute world position: offset * 32 + corner
   00433c5c: SHL AX,0x5
   00433c64: ADD AX,word ptr [ESP + 0x20]      ; + corner_x
   ```

4. **Get terrain height at rubble position:**
   ```asm
   00433c82: CALL Terrain_InterpolateHeight     ; 0x4E8E50
   ```

5. **Create rubble Scenery object via command buffer:**
   ```asm
   ; cmd[0x00] = (EBP == 1) ? 1 : 0 (first rubble gets special flag)
   00433c95: CMP EBP,0x1
   00433c98: SBB EAX,EAX
   00433c9b: NEG EAX
   00433c9d: MOV dword ptr [ECX],EAX           ; cmd[0] = rotation flag

   00433ca5: MOV dword ptr [ECX + 0x4],0x0     ; cmd[4] = 0
   00433cb2: MOV dword ptr [ECX + 0x8],EBX     ; cmd[8] = rubble subtype + 1
   00433cbf: MOV dword ptr [ECX + 0xc],0x1     ; cmd[0xC] = 1 (link to grid)
   00433ccd: MOV dword ptr [ECX + 0x10],0x0    ; cmd[0x10] = 0
   00433cd4: ADD dword ptr [0x0087a9db],0x14   ; advance buffer
   00433cdb: MOV byte ptr [0x0087a9d2],0x1     ; set creation flag

   ; Create Scenery/10 (rubble) object
   00433ce8: PUSH 0xa                          ; subtype = 10 (rubble)
   00433cea: PUSH 0x5                          ; model = Scenery (5)
   00433cea: CALL Object_Create                ; 0x4AFC70
   ```

6. **After rubble creation, set parent building flag:**
   ```asm
   00433cfa: OR dword ptr [ESI + 0x10],0x10    ; set flag 0x10 on original building
   00433d04: PUSH 0x87                         ; animation/effect ID
   00433d0c: PUSH EAX                          ; rubble object
   00433d1c: CALL 0x004bfb60                   ; set rubble animation
   ```

7. **Search cell for Person objects of same tribe → force flee:**
   ```asm
   ; Convert rubble position to cell address
   ; Walk cell's object linked list
   00433d6a: CMP byte ptr [EBX + 0x2a],0x1    ; model_type == Person (1)?
   00433d73: CMP byte ptr [EBX + 0x2f],AL     ; same tribe as building?
   ; Check person subtype properties for can-flee flag
   00433d83: TEST byte ptr [EDX*0x2 + 0x59fe71],0x1
   00433d8d: TEST byte ptr [EBX + 0xe],0x10   ; already fleeing?
   00433d91: JNZ skip_flee

   ; Set person to flee state (0x1A)
   00433d93: MOV AL,byte ptr [EBX + 0x2c]     ; save current state
   00433d97: MOV byte ptr [EBX + 0x7d],AL     ; store in obj+0x7D (saved state)
   00433d9a: CALL Object_ClearStateByType_Stub ; 0x4AFAC0
   00433da2: MOV byte ptr [EBX + 0x2c],0x1a   ; state = 0x1A (fleeing)
   00433da7: CALL Object_SetStateByType        ; 0x4AFA10
   ```

8. **PRNG-based flee timer:**
   ```asm
   ; PRNG: val = val*0x41C64E6D + 0x24DF; val = ROR(val, 13)
   00433daf: MOV EAX,[0x00885710]              ; PRNG state
   00433db6: LEA EDX,[EAX + EAX*0x8]
   00433db9: LEA EAX,[ECX + EDX*0x8]
   00433dbc: LEA EAX,[ECX + EAX*0x4]
   00433dbf: SHL EAX,0x2
   00433dc2: LEA EAX,[ECX + EAX*0x8]
   00433dc5: ADD EAX,0x24df
   00433dca: MOV [0x00885710],EAX
   00433dd3: ROR dword ptr [ESP + 0x14],0xd
   ; flee_timer = (PRNG & 0x7) + 8 → range 8-15
   00433de1: AND AL,0x7
   00433de3: ADD AL,0x8
   00433de5: MOV byte ptr [EBX + 0xa4],AL     ; person flee timer
   ```

**Footprint adjacent position table (at footprint+0x1A, 6 entries × 3 bytes):**
Each entry: [x_cell_offset, rubble_subtype_index, z_cell_offset].
All zeros = skip this slot. These define positions around the building where rubble spawns.

### BPLC.31 — Building_MarkFootprintBuildingFlags (0x0042EF80)

Marks all cells within a building's footprint with the building flag (0x10) in cell[0],
and computes the altitude control value for each cell.

**Input:** object pointer (ESP+0x4)

**Key operations:**

1. **Get footprint data:**
   ```asm
   0042ef87: MOVSX EAX,word ptr [EDX + 0x33]  ; obj+0x33 = shape index
   ; Compute shape_data_index * 54 (=shape_index*6*9)
   0042ef94: MOV ECX,dword ptr [0x0087e459]    ; shape data table
   0042ef9d: MOVSX EBP,byte ptr [EBX + ECX*0x1 + 0x2c]  ; footprint_idx
   ; If footprint_idx == 0, use 1 instead (minimum 1-cell footprint)
   0042efa4: JNZ skip_default
   0042efa6: MOV EBP,0x1
   ```

2. **Compute corner position (same as other footprint functions):**
   ```asm
   ; footprint_ptr = footprint_table[footprint_idx * 48]
   0042efab: SHL EBP,0x4                      ; * 16
   0042efb2: LEA EBP,[EBP + EBP*0x2]          ; * 3 → total * 48
   0042efb6: SHR AX,0x8                       ; cell_x from position
   0042efca: SHR AX,0x8                       ; cell_z from position
   ; Subtract origin: cell -= footprint.origin
   0042efe7: SUB byte ptr [ESP + 0x12],DL     ; x -= origin_x
   0042efeb: SUB byte ptr [ESP + 0x13],AL     ; z -= origin_z
   ```

3. **Iterate width × height cells, for each occupied cell (mask & 0x01):**
   ```asm
   0042f01e: TEST byte ptr [EBX],0x1          ; footprint mask[cell] & 0x01
   0042f021: JZ skip_cell

   ; Compute cell address from position
   0042f040: LEA EDI,[EAX*0x4 + 0x88897c]    ; cell_ptr

   ; Set building flag
   0042f048: OR dword ptr [EDI],0x10          ; cell[0] |= 0x10 (has building)

   ; Get altitude from Cell_GetBuildingAltitude
   0042f04b: CALL Cell_GetBuildingAltitude     ; 0x4EB260
   0042f053: CMP EAX,0xf
   0042f058: MOV EAX,0xf                      ; cap at 15

   ; Write altitude to cell+0x0E low nibble
   0042f05d: MOV CL,byte ptr [EDI + 0xe]     ; cell+0x0E
   0042f060: AND CL,0xf0                      ; clear low nibble
   0042f063: OR CL,AL                         ; set altitude (0-15)
   0042f065: MOV byte ptr [EDI + 0xe],CL
   ```

4. **After marking, notify render system (max dimension / 2):**
   ```asm
   ; max_dim = max(height/2, width/2)
   0042f09a: SAR EAX,0x1                      ; height / 2
   0042f09d: SAR ECX,0x1                      ; width / 2
   0042f0a0: CMP EAX,ECX
   0042f0a4: MOV EAX,ECX                      ; take max
   0042f0ac: CALL 0x00487870                   ; notify render (position, radius)
   ```

### BPLC.32 — Terrain_QueueFlattenArea (0x004E8300)

Queues a rectangular area of terrain cells for height flattening. Uses a ring buffer
of up to 1024 entries with a deduplication grid.

**Input:**
- ESP+0x04: position (packed x,z cell coords)
- ESP+0x08: radius (half-width of area to flatten)
- ESP+0x0C: flatten mode/height target

**Key data structures:**

| Address | Name | Description |
|---------|------|-------------|
| 0x00972840 | g_flatten_queue_write_idx | Write index into ring buffer (0-1023) |
| 0x00972844 | g_flatten_queue_dup_count | Count of deduplicated entries |
| 0x00972C50 | g_flatten_queue_positions | Ring buffer: 1024 × 2-byte position entries |
| 0x00972848 | g_flatten_queue_modes | Ring buffer: 1024 × 1-byte mode entries |
| 0x0096E838 | g_flatten_total_count | Total flatten operations queued |
| 0x0096E840 | g_flatten_dedup_grid | 128×128 byte grid for deduplication |
| 0x00972C48 | g_flatten_recursion_guard | Prevents recursive re-entry |

**Logic:**

1. **Compute area dimensions:**
   ```asm
   004e8300: MOVSX EAX,word ptr [ESP + 0x8]   ; radius
   004e8308: LEA ECX,[EAX*0x2 + 0x1]          ; side_length = radius*2 + 1
   ; Subtract radius from position to get corner
   004e8322: ADD AL,AL                         ; radius * 2
   004e832a: SUB byte ptr [ESP + 0x14],AL     ; corner_x = pos_x - radius*2
   004e832e: SUB byte ptr [ESP + 0x15],AL     ; corner_z = pos_z - radius*2
   ```

2. **For each cell in the area (side_length × side_length):**
   ```asm
   ; Check deduplication grid
   004e8373: CMP byte ptr [ECX + EBX*0x1 + 0x96e840],0x0
   004e837b: JNZ already_queued

   ; Write to ring buffer
   004e8386: MOV word ptr [EDI*0x2 + 0x972c50],AX  ; position
   004e838e: MOV byte ptr [EDI + 0x972848],CL      ; mode
   004e8399: INC EDI                                ; advance write index

   ; Mark dedup grid
   004e83b1: MOV byte ptr [EAX + EDX*0x1 + 0x96e840],0x1
   ```

3. **Ring buffer overflow handling:**
   ```asm
   004e83c7: CMP EDI,0x400                    ; buffer full (1024)?
   004e83cd: JNZ continue
   004e83cf: CALL 0x004e8450                   ; flush/process buffer
   ```

4. **Recursive call for mode 0x40:**
   ```asm
   004e8404: CMP word ptr [ESP + 0x24],0x40   ; mode == 0x40?
   004e840a: JNZ done
   004e840c: CALL 0x004e8450                   ; flush first
   ; Guard against recursion
   004e8418: CMP byte ptr [0x00972c48],0x0
   004e8427: MOV byte ptr [0x00972c48],0x1    ; set guard
   004e8430: CALL Terrain_QueueFlattenArea     ; recurse with same params
   004e8435: MOV byte ptr [0x00972c48],0x0    ; clear guard
   ```

### BPLC.33 — Cell Grid Structure (Consolidated)

The cell grid at 0x88897C contains 128×128 cells (16,384 total), each 16 (0x10) bytes.
Total grid size: 262,144 bytes (256 KB).

**Cell structure (16 bytes):**

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| +0x00 | 4 | flags | Master flag dword |
| +0x04 | 2 | terrain_height | Terrain height at this cell |
| +0x06 | 2 | object_list_head | First object ID in cell's linked list |
| +0x08 | 2 | building_ref | Building object ID (10-bit) + flags |
| +0x0A | 1 | (unknown) | |
| +0x0B | 1 | ownership | Low 4 bits = tribe+1 (0=unowned) |
| +0x0C | 1 | terrain_type | Low 4 bits = terrain type index |
| +0x0D | 1 | (unknown) | |
| +0x0E | 1 | altitude_ctrl | Low 4 bits = building altitude (0-15) |
| +0x0F | 1 | (unknown) | |

**Flag bits in cell[+0x00]:**

| Bit(s) | Mask | Set by | Description |
|--------|------|--------|-------------|
| 0 | 0x01 | Terrain | Diagonal split flag (affects height interpolation) |
| 1 | 0x02 | Building | Has building flag (cell+0x01 bit 2) |
| 2 | 0x04 | Object | Object presence (blocks normal building placement) |
| 4 | 0x10 | Building | Building footprint cell (set by MarkFootprintBuildingFlags) |
| 9 | 0x200 | Building | Completed building cell (blocks special building placement) |
| 20 | 0x100000 | Building | Destroying flag |

**Cell address computation:**
```
cell_x = (world_x >> 8) & 0xFE
cell_z = (world_z >> 8) & 0xFE
index = (cell_x * 2) | (cell_z << 1 & 0xFE00)  [packed]
cell_ptr = 0x88897C + index * 4
```

**Placement validation flag checks:**
- Normal buildings: cell[0] & 0x100004 must be zero
- Special buildings: cell[0] & 0x100204 must be zero
- Building_ValidatePlacement: cell[0] & 0x206 must be zero (different check)

### BPLC.34 — Complete Function Reference (All Iterations)

| Address | Name | Iter | Description |
|---------|------|------|-------------|
| 0x0040C330 | Level_LoadAndCreateObjects | 1 | Main level loader, creates all objects from DAT file |
| 0x0040D420 | Level_PostCreateUnit | 1 | Post-creation init per unit type |
| 0x0040DFC0 | Level_SpawnBuildingsFromGenerals | 2 | Post-load: spawn Prison from General/6 markers |
| 0x0042E230 | Building_Init | 1 | Building subtype dispatch (19 types) |
| 0x0042E430 | Building_SetState | 2 | 6-state building state machine |
| 0x0042E980 | Building_InitFromType | 1 | Core building initialization |
| 0x0042ED70 | Building_MarkFootprintCells | 1 | Mark cells as building-occupied (owner, flags) |
| 0x0042EF80 | Building_MarkFootprintBuildingFlags | 3 | Mark cells with building flag + altitude |
| 0x0042F0C0 | Building_UpdateFootprint | 1 | Compute corner from rotation + shape |
| 0x0042F2A0 | Building_FlattenTerrain | 1 | Flatten terrain under building footprint |
| 0x0042FD70 | Building_OnConstructionComplete | 3 | Construction done → set shape, entrance, timers |
| 0x00433BB0 | Building_OnDestroy | 3 | Spawn rubble, cause persons to flee |
| 0x00436340 | Building_ResetFireEffects | 2 | Reset fire effects on footprint cells |
| 0x004364E0 | Building_InitModelSelector | 2 | Select the OBJS model from type data and PRNG |
| 0x00499EB0 | Object_IsValidPosition | 3 | Validate object position on terrain |
| 0x0049B9B0 | Shape_LoadBankData | 2 | Load shape bank + patch pointers |
| 0x0049BBA0 | Shape_LoadDatFile | 2 | Load SHAPES.DAT footprint data |
| 0x0049BC40 | Shape_PatchPointers | 2 | Fix up shape data internal pointers |
| 0x004AF950 | Object_InitByType | 1 | Dispatch to type-specific init (11 types) |
| 0x004AFA10 | Object_SetStateByType | 1 | Dispatch to type-specific state set |
| 0x004AFAC0 | Object_ClearStateByType_Stub | 1 | No-op (single RET) |
| 0x004AFC70 | Object_Create | 1 | Allocate object from free list |
| 0x004B0840 | Object_LinkToCell | 1 | Link object into cell grid |
| 0x004B0950 | Object_MoveToPosition | 1 | Move object to new cell position |
| 0x004B0AD0 | Object_SetShapeFromType | 2 | Set shape from type properties table |
| 0x004B5990 | Building_ValidatePlacement | 1 | Check if placement is legal (runtime only) |
| 0x004E8300 | Terrain_QueueFlattenArea | 3 | Queue terrain area for flattening |
| 0x004E8E50 | Terrain_InterpolateHeight | 2 | Bilinear height interpolation |
| 0x004EB260 | Cell_GetBuildingAltitude | 3 | Get building altitude from cell |

**Data addresses:**

| Address | Name | Iter | Description |
|---------|------|------|-------------|
| 0x005A0014 | (building_type_table+0x14) | 1 | Shape IDs per building type |
| 0x005A003D | (building_type_table+0x3D) | 3 | Base altitude per building type |
| 0x005A0050 | (building_type_table+0x50) | 1 | Behavior flags per building type |
| 0x005A072D | (subtype_properties) | 3 | Per-subtype flags (bit 0x01 = special placement) |
| 0x005A0974 | (height_limit_table) | 3 | Max height per limit index |
| 0x005A3038 | (terrain_type_props) | 3 | Terrain type properties (placement masks) |
| 0x005A7D78 | g_building_footprint_table_ptr | 1 | SHAPES.DAT footprint table pointer |
| 0x0087A9D2 | g_object_create_flag | 1 | Command buffer pending flag |
| 0x0087A9DB | g_object_create_cmd_buf_ptr | 1 | Creation command buffer pointer |
| 0x0087E459 | g_building_shape_data_ptr | 1 | Shape/rotation data table pointer |
| 0x008788B4 | g_object_freelist_a | 1 | Primary free object list |
| 0x008788B8 | g_object_freelist_b | 1 | Secondary free object list |
| 0x008788BC | g_object_active_list | 1 | Active object linked list |
| 0x00878928 | g_object_table | 3 | Object pointer lookup table (ID → pointer) |
| 0x0088897C | g_cell_grid | 3 | 128×128 cell grid (16 bytes per cell) |
| 0x00885710 | g_prng_state | 3 | PRNG state for flee timers |
| 0x00598170 | g_shape_count | 2 | Number of shape entries |
| 0x00972840 | g_flatten_queue_write_idx | 3 | Terrain flatten queue write index |
| 0x0096E840 | g_flatten_dedup_grid | 3 | Terrain flatten deduplication grid |
