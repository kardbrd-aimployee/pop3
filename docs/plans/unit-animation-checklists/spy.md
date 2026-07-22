# Spy animation checklist

Native subtype: `5`

Primary mechanics: training, movement, disguise and infiltration, combat, vehicles, and water

Extracted original-game sequences: `23`

Shared rules: [person state and animation checklist](../person-state-animation-checklist.md)

## Current Rust state adapter

| Check | Exact `PersonState` values | Row and spy ID | Verification |
|---|---|---|---|
| [ ] | `Idle`, `InsideTraining`, `InShield`, `WaitingAtReincPillar` | Idle row 0, ID 18 | Capture open |
| [ ] | `Moving`, `Wander`, `GoToPoint`, `FollowPath`, `GoToMarker`, `WaitForPath`, `WaitAtMarker`, `EnterBuilding`, `WaitOutside`, `Training`, `Housing`, `Gathering`, `Spawning`, `BeingConverted`, `WaitingAfterConvert`, `WaitingForBoat`, `Placeholder`, `GetOffBoat`, `EnteringVehicle`, `Teleporting`, `InternalState`, `InShieldIdle` | Walk row 1, ID 24; zero speed falls back to ID 18 | Mixed verified and provisional mappings |
| [ ] | `InsideBuilding`, `InTraining`, `Fighting` | Action row 3, ID 35 | Handler overrides open |
| [ ] | `Dying`, `Dead`, `BeingSacrificed` | Die row 6, ID 30 | Sacrifice mapping open |
| [ ] | `Celebrating` | Celebrate row 7, ID 41 | Capture open |
| [ ] | `GatheringWood` | Work row 13, ID 76 | Mechanic assignment open |
| [ ] | `Drowning`, `WaitingInWater` | Swim row 16, ID 86 | Waterline capture open |
| [ ] | `CarryingWood` | Carry row 18, ID 91 | Mechanic assignment open |
| [ ] | `Building` | Walk row 1, ID 24 | Spies must not receive brave construction jobs |
| [ ] | `SitDown` | Sit row 21, ID 134 | Three other variants remain unselected |
| [ ] | `Fleeing`, `Preaching`, `ExitingVehicle` | Run row 25, ID 159 | Handler use open |

## State mapping

| Check | States or mechanic | Planned sequence | Status |
|---|---|---|---|
| [ ] | Idle-class states | Idle row 0, ID 18<br><img src="../../images/person-animation-plan/spy-idle-id18.png" width="276" alt="Spy idle frames, ID 18"> | Cadence capture open |
| [ ] | Moving, path, marker, and entrance travel | Walk row 1, ID 24<br><img src="../../images/person-animation-plan/spy-walk-id24.png" width="288" alt="Spy walk frames, ID 24"> | Runtime mapping exists |
| [ ] | Fighting and training actions | Action row 3, ID 35<br><img src="../../images/person-animation-plan/spy-action-id35.png" width="360" alt="Spy action frames, ID 35"> | Attack timing open |
| [ ] | Disguise and infiltration | Unassigned | Capture tribe-color, body-layer, and reveal transitions |
| [ ] | Special row ownership | Native table points ID 101 at subtype 5; extractor labels ID 101 as firewarrior | Resolve the table/extractor conflict before use |
| [ ] | Dying and dead hold | Die row 6, ID 30<br><img src="../../images/person-animation-plan/spy-die-id30.png" width="324" alt="Spy death frames, ID 30"> | One-shot and final-frame rules open |
| [ ] | Fleeing and fast exit | Run row 25, ID 159<br><img src="../../images/person-animation-plan/spy-run-id159.png" width="300" alt="Spy run frames, ID 159"> | Exit capture open |
| [ ] | Drowning and waiting in water | Swim row 16, ID 86<br><img src="../../images/person-animation-plan/spy-swim-id86.png" width="330" alt="Spy swim frames, ID 86"> | Waterline offset open |
| [ ] | SitDown | IDs 134, 139, 144, and 149 | Variant selector open |
| [ ] | Vehicle entry, travel, and exit | Walk, vehicle ID 81, ride ID 113, then run | Transition capture open |
| [ ] | Carry, dig, build, and work rows | Extracted but unassigned | Do not inherit brave construction rules |
| [ ] | Spawning, sacrifice, conversion, teleport, and internal states | Unassigned | Handler evidence required |

## Extracted sequence inventory

| Check | Native row or sequence | Logical ID | Original frames |
|---|---|---:|---|
| [ ] | Idle | 18 | <img src="../../images/person-animation-plan/spy-idle-id18.png" width="276" alt="Spy idle frames, ID 18"> |
| [ ] | Walk | 24 | <img src="../../images/person-animation-plan/spy-walk-id24.png" width="288" alt="Spy walk frames, ID 24"> |
| [ ] | Die | 30 | <img src="../../images/person-animation-plan/spy-die-id30.png" width="324" alt="Spy death frames, ID 30"> |
| [ ] | Action | 35 | <img src="../../images/person-animation-plan/spy-action-id35.png" width="360" alt="Spy action frames, ID 35"> |
| [ ] | Celebrate | 41 | <img src="../../images/person-animation-plan/spy-celebrate-id41.png" width="330" alt="Spy celebrate frames, ID 41"> |
| [ ] | Spell idle | 46 | <img src="../../images/person-animation-plan/spy-spell-idle-id46.png" width="88" alt="Spy spell-idle frame, ID 46"> |
| [ ] | Spell walk | 51 | <img src="../../images/person-animation-plan/spy-spell-walk-id51.png" width="300" alt="Spy spell-walk frames, ID 51"> |
| [ ] | Work 1 | 56 | <img src="../../images/person-animation-plan/spy-work1-id56.png" width="504" alt="Spy work-one frames, ID 56"> |
| [ ] | Work 2 | 61 | <img src="../../images/person-animation-plan/spy-work2-id61.png" width="378" alt="Spy work-two frames, ID 61"> |
| [ ] | Work 3 | 66 | <img src="../../images/person-animation-plan/spy-work3-id66.png" width="490" alt="Spy work-three frames, ID 66"> |
| [ ] | Work 4 | 71 | <img src="../../images/person-animation-plan/spy-work4-id71.png" width="96" alt="Spy work-four frame, ID 71"> |
| [ ] | Work 5 | 76 | <img src="../../images/person-animation-plan/spy-work5-id76.png" width="372" alt="Spy work-five frames, ID 76"> |
| [ ] | Vehicle | 81 | <img src="../../images/person-animation-plan/spy-vehicle-id81.png" width="330" alt="Spy vehicle frames, ID 81"> |
| [ ] | Swim | 86 | <img src="../../images/person-animation-plan/spy-swim-id86.png" width="330" alt="Spy swim frames, ID 86"> |
| [ ] | Carry | 91 | <img src="../../images/person-animation-plan/spy-carry-id91.png" width="276" alt="Spy carry frames, ID 91"> |
| [ ] | Ride | 113 | <img src="../../images/person-animation-plan/spy-ride-id113.png" width="330" alt="Spy ride frames, ID 113"> |
| [ ] | Dig / internal 1 | 118 | <img src="../../images/person-animation-plan/spy-dig-id118.png" width="512" alt="Spy dig frames, ID 118"> |
| [ ] | Build / internal 2 | 123 | <img src="../../images/person-animation-plan/spy-build-id123.png" width="350" alt="Spy build frames, ID 123"> |
| [ ] | Sit 1 | 134 | <img src="../../images/person-animation-plan/spy-sit1-id134.png" width="384" alt="Spy sit-one frames, ID 134"> |
| [ ] | Sit 2 | 139 | <img src="../../images/person-animation-plan/spy-sit2-id139.png" width="486" alt="Spy sit-two frames, ID 139"> |
| [ ] | Sit 3 | 144 | <img src="../../images/person-animation-plan/spy-sit3-id144.png" width="350" alt="Spy sit-three frames, ID 144"> |
| [ ] | Sit 4 | 149 | <img src="../../images/person-animation-plan/spy-sit4-id149.png" width="276" alt="Spy sit-four frames, ID 149"> |
| [ ] | Run | 159 | <img src="../../images/person-animation-plan/spy-run-id159.png" width="300" alt="Spy run frames, ID 159"> |

## Acceptance

- [ ] The renderer keeps subtype `5` through each state transition.
- [ ] The resolved VSTART and render type match the logical ID.
- [ ] The Rust frame count and order match the strip.
- [ ] Training produces subtype `5` at the building entrance.
- [ ] Disguise changes tribe presentation without changing the underlying subtype.
- [ ] A binary audit resolves logical ID 101 ownership.
