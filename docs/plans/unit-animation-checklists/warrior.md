# Warrior animation checklist

Native subtype: `3`

Primary mechanics: training, movement, melee combat, vehicles, and water

Extracted original-game sequences: `23`

Shared rules: [person state and animation checklist](../person-state-animation-checklist.md)

## Current Rust state adapter

| Check | Exact `PersonState` values | Row and warrior ID | Verification |
|---|---|---|---|
| [ ] | `Idle`, `InsideTraining`, `InShield`, `WaitingAtReincPillar` | Idle row 0, ID 16 | Capture open |
| [ ] | `Moving`, `Wander`, `GoToPoint`, `FollowPath`, `GoToMarker`, `WaitForPath`, `WaitAtMarker`, `EnterBuilding`, `WaitOutside`, `Training`, `Housing`, `Gathering`, `Spawning`, `BeingConverted`, `WaitingAfterConvert`, `WaitingForBoat`, `Placeholder`, `GetOffBoat`, `EnteringVehicle`, `Teleporting`, `InternalState`, `InShieldIdle` | Walk row 1, ID 22; zero speed falls back to ID 16 | Mixed verified and provisional mappings |
| [ ] | `InsideBuilding`, `InTraining`, `Fighting` | Action row 3, ID 33 | Handler overrides open |
| [ ] | `Dying`, `Dead`, `BeingSacrificed` | Die row 6, ID 28 | Sacrifice mapping open |
| [ ] | `Celebrating` | Celebrate row 7, ID 39 | Capture open |
| [ ] | `GatheringWood` | Work row 13, ID 74 | Mechanic assignment open |
| [ ] | `Drowning`, `WaitingInWater` | Swim row 16, ID 84 | Waterline capture open |
| [ ] | `CarryingWood` | Carry row 18, ID 89 | Mechanic assignment open |
| [ ] | `Building` | Walk row 1, ID 22 | Warriors must not receive brave construction jobs |
| [ ] | `SitDown` | Sit row 21, ID 132 | Three other variants remain unselected |
| [ ] | `Fleeing`, `Preaching`, `ExitingVehicle` | Run row 25, ID 157 | Handler use open |

## State mapping

| Check | States or mechanic | Planned sequence | Status |
|---|---|---|---|
| [ ] | Idle-class states | Idle row 0, ID 16<br><img src="../../images/person-animation-plan/warrior-idle-id16.png" width="276" alt="Warrior idle frames, ID 16"> | Cadence capture open |
| [ ] | Moving, path, marker, and entrance travel | Walk row 1, ID 22<br><img src="../../images/person-animation-plan/warrior-walk-id22.png" width="312" alt="Warrior walk frames, ID 22"> | Runtime mapping exists |
| [ ] | Fighting and training actions | Action row 3, ID 33<br><img src="../../images/person-animation-plan/warrior-action-id33.png" width="360" alt="Warrior action frames, ID 33"> | Attack and hit timing open |
| [ ] | Dying and dead hold | Die row 6, ID 28<br><img src="../../images/person-animation-plan/warrior-die-id28.png" width="324" alt="Warrior death frames, ID 28"> | One-shot and final-frame rules open |
| [ ] | Fleeing and fast exit | Run row 25, ID 157<br><img src="../../images/person-animation-plan/warrior-run-id157.png" width="300" alt="Warrior run frames, ID 157"> | Exit capture open |
| [ ] | Drowning and waiting in water | Swim row 16, ID 84<br><img src="../../images/person-animation-plan/warrior-swim-id84.png" width="330" alt="Warrior swim frames, ID 84"> | Waterline offset open |
| [ ] | SitDown | IDs 132, 137, 142, and 147 | Variant selector open |
| [ ] | Vehicle entry, travel, and exit | Walk, vehicle ID 79, ride ID 111, then run | Transition capture open |
| [ ] | Carry, dig, build, and work rows | Extracted but unassigned | Do not inherit brave construction rules |
| [ ] | Spawning, sacrifice, conversion, teleport, and internal states | Unassigned | Handler evidence required |

## Extracted sequence inventory

| Check | Native row or sequence | Logical ID | Original frames |
|---|---|---:|---|
| [ ] | Idle | 16 | <img src="../../images/person-animation-plan/warrior-idle-id16.png" width="276" alt="Warrior idle frames, ID 16"> |
| [ ] | Walk | 22 | <img src="../../images/person-animation-plan/warrior-walk-id22.png" width="312" alt="Warrior walk frames, ID 22"> |
| [ ] | Die | 28 | <img src="../../images/person-animation-plan/warrior-die-id28.png" width="324" alt="Warrior death frames, ID 28"> |
| [ ] | Action | 33 | <img src="../../images/person-animation-plan/warrior-action-id33.png" width="360" alt="Warrior action frames, ID 33"> |
| [ ] | Celebrate | 39 | <img src="../../images/person-animation-plan/warrior-celebrate-id39.png" width="330" alt="Warrior celebrate frames, ID 39"> |
| [ ] | Spell idle | 44 | <img src="../../images/person-animation-plan/warrior-spell-idle-id44.png" width="88" alt="Warrior spell-idle frame, ID 44"> |
| [ ] | Spell walk | 49 | <img src="../../images/person-animation-plan/warrior-spell-walk-id49.png" width="300" alt="Warrior spell-walk frames, ID 49"> |
| [ ] | Work 1 | 54 | <img src="../../images/person-animation-plan/warrior-work1-id54.png" width="520" alt="Warrior work-one frames, ID 54"> |
| [ ] | Work 2 | 59 | <img src="../../images/person-animation-plan/warrior-work2-id59.png" width="392" alt="Warrior work-two frames, ID 59"> |
| [ ] | Work 3 | 64 | <img src="../../images/person-animation-plan/warrior-work3-id64.png" width="574" alt="Warrior work-three frames, ID 64"> |
| [ ] | Work 4 | 69 | <img src="../../images/person-animation-plan/warrior-work4-id69.png" width="96" alt="Warrior work-four frame, ID 69"> |
| [ ] | Work 5 | 74 | <img src="../../images/person-animation-plan/warrior-work5-id74.png" width="372" alt="Warrior work-five frames, ID 74"> |
| [ ] | Vehicle | 79 | <img src="../../images/person-animation-plan/warrior-vehicle-id79.png" width="350" alt="Warrior vehicle frames, ID 79"> |
| [ ] | Swim | 84 | <img src="../../images/person-animation-plan/warrior-swim-id84.png" width="330" alt="Warrior swim frames, ID 84"> |
| [ ] | Carry | 89 | <img src="../../images/person-animation-plan/warrior-carry-id89.png" width="276" alt="Warrior carry frames, ID 89"> |
| [ ] | Ride | 111 | <img src="../../images/person-animation-plan/warrior-ride-id111.png" width="350" alt="Warrior ride frames, ID 111"> |
| [ ] | Dig / internal 1 | 116 | <img src="../../images/person-animation-plan/warrior-dig-id116.png" width="512" alt="Warrior dig frames, ID 116"> |
| [ ] | Build / internal 2 | 121 | <img src="../../images/person-animation-plan/warrior-build-id121.png" width="370" alt="Warrior build frames, ID 121"> |
| [ ] | Sit 1 | 132 | <img src="../../images/person-animation-plan/warrior-sit1-id132.png" width="384" alt="Warrior sit-one frames, ID 132"> |
| [ ] | Sit 2 | 137 | <img src="../../images/person-animation-plan/warrior-sit2-id137.png" width="486" alt="Warrior sit-two frames, ID 137"> |
| [ ] | Sit 3 | 142 | <img src="../../images/person-animation-plan/warrior-sit3-id142.png" width="378" alt="Warrior sit-three frames, ID 142"> |
| [ ] | Sit 4 | 147 | <img src="../../images/person-animation-plan/warrior-sit4-id147.png" width="276" alt="Warrior sit-four frames, ID 147"> |
| [ ] | Run | 157 | <img src="../../images/person-animation-plan/warrior-run-id157.png" width="300" alt="Warrior run frames, ID 157"> |

## Acceptance

- [ ] The renderer keeps subtype `3` through each state transition.
- [ ] The resolved VSTART and render type match the logical ID.
- [ ] The Rust frame count and order match the strip.
- [ ] Training produces subtype `3` at the building entrance.
- [ ] Original-game and Rust captures agree on combat cadence and movement.
