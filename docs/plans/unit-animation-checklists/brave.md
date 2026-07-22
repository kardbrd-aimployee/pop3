# Brave animation checklist

Native subtype: `2`

Primary mechanics: movement, settlement construction, wood gathering, combat, vehicles, and water

Extracted original-game sequences: `24`

Shared rules: [person state and animation checklist](../person-state-animation-checklist.md)

## Current Rust state adapter

| Check | Exact `PersonState` values | Row and brave ID | Verification |
|---|---|---|---|
| [ ] | `Idle`, `InsideTraining`, `InShield`, `WaitingAtReincPillar` | Idle row 0, ID 15 | Capture open |
| [ ] | `Moving`, `Wander`, `GoToPoint`, `FollowPath`, `GoToMarker`, `WaitForPath`, `WaitAtMarker`, `EnterBuilding`, `WaitOutside`, `Training`, `Housing`, `Gathering`, `Spawning`, `BeingConverted`, `WaitingAfterConvert`, `WaitingForBoat`, `Placeholder`, `GetOffBoat`, `EnteringVehicle`, `Teleporting`, `InternalState`, `InShieldIdle` | Walk row 1, ID 21; zero speed falls back to ID 15 | Mixed verified and provisional mappings |
| [ ] | `InsideBuilding`, `InTraining`, `Fighting` | Action row 3, ID 32 | Handler overrides open |
| [ ] | `Dying`, `Dead`, `BeingSacrificed` | Die row 6, ID 27 | Sacrifice mapping open |
| [ ] | `Celebrating` | Celebrate row 7, ID 38 | Capture open |
| [ ] | `GatheringWood` | Work row 13, ID 73 | Tree-work capture open |
| [ ] | `Drowning`, `WaitingInWater` | Swim row 16, ID 83 | Waterline capture open |
| [ ] | `CarryingWood` | Carry row 18, ID 88 | Known runtime mismatch |
| [ ] | `Building` | Walk row 1, ID 21 plus construction subphase motion | Known missing hop in prior app |
| [ ] | `SitDown` | Sit row 21, ID 131 | Three other variants remain unselected |
| [ ] | `Fleeing`, `Preaching`, `ExitingVehicle` | Run row 25, ID 156 | Preaching target action open |

## State mapping

| Check | States or mechanic | Planned sequence | Status |
|---|---|---|---|
| [ ] | Idle-class states | Idle row 0, ID 15<br><img src="../../images/person-animation-plan/brave-idle-id15.png" width="276" alt="Brave idle frames, ID 15"> | Cadence capture open |
| [ ] | Moving, path, marker, and entrance travel | Walk row 1, ID 21<br><img src="../../images/person-animation-plan/brave-walk-id21.png" width="288" alt="Brave walk frames, ID 21"> | Runtime mapping exists |
| [ ] | Foundation flattening | Walk row 1, ID 21 plus render-height hop<br><img src="../../images/person-animation-plan/brave-walk-id21.png" width="288" alt="Brave walk frames, ID 21"> | Known missing motion in prior app |
| [ ] | Tree work | Work row 13 candidate, ID 73<br><img src="../../images/person-animation-plan/brave-work5-id73.png" width="372" alt="Brave work-five frames, ID 73"> | Native gameplay capture required |
| [ ] | Carry wood | Carry row 18 candidate, ID 88<br><img src="../../images/person-animation-plan/brave-carry-id88.png" width="276" alt="Brave carry frames, ID 88"> | Runtime body composition mismatch |
| [ ] | Fighting and training actions | Action row 3, ID 32<br><img src="../../images/person-animation-plan/brave-action-id32.png" width="360" alt="Brave action frames, ID 32"> | Foundation work must not use this row |
| [ ] | Dying and dead hold | Die row 6, ID 27<br><img src="../../images/person-animation-plan/brave-die-id27.png" width="324" alt="Brave death frames, ID 27"> | One-shot and final-frame rules open |
| [ ] | Fleeing and fast exit | Run row 25, ID 156<br><img src="../../images/person-animation-plan/brave-run-id156.png" width="300" alt="Brave run frames, ID 156"> | Exit capture open |
| [ ] | Drowning and waiting in water | Swim row 16, ID 83<br><img src="../../images/person-animation-plan/brave-swim-id83.png" width="330" alt="Brave swim frames, ID 83"> | Waterline offset open |
| [ ] | SitDown | IDs 131, 136, 141, and 146 | Variant selector open |
| [ ] | Vehicle entry, travel, and exit | Walk, vehicle ID 78, ride ID 110, then run | Transition capture open |
| [ ] | Spawning, sacrifice, conversion, teleport, and internal states | Unassigned | Handler evidence required |

## Extracted sequence inventory

| Check | Native row or sequence | Logical ID | Original frames |
|---|---|---:|---|
| [ ] | Idle | 15 | <img src="../../images/person-animation-plan/brave-idle-id15.png" width="276" alt="Brave idle frames, ID 15"> |
| [ ] | Walk | 21 | <img src="../../images/person-animation-plan/brave-walk-id21.png" width="288" alt="Brave walk frames, ID 21"> |
| [ ] | Die | 27 | <img src="../../images/person-animation-plan/brave-die-id27.png" width="324" alt="Brave death frames, ID 27"> |
| [ ] | Action | 32 | <img src="../../images/person-animation-plan/brave-action-id32.png" width="360" alt="Brave action frames, ID 32"> |
| [ ] | Celebrate | 38 | <img src="../../images/person-animation-plan/brave-celebrate-id38.png" width="330" alt="Brave celebrate frames, ID 38"> |
| [ ] | Spell idle | 43 | <img src="../../images/person-animation-plan/brave-spell-idle-id43.png" width="88" alt="Brave spell-idle frame, ID 43"> |
| [ ] | Spell walk | 48 | <img src="../../images/person-animation-plan/brave-spell-walk-id48.png" width="300" alt="Brave spell-walk frames, ID 48"> |
| [ ] | Work 1 | 53 | <img src="../../images/person-animation-plan/brave-work1-id53.png" width="504" alt="Brave work-one frames, ID 53"> |
| [ ] | Work 2 | 58 | <img src="../../images/person-animation-plan/brave-work2-id58.png" width="378" alt="Brave work-two frames, ID 58"> |
| [ ] | Work 3 | 63 | <img src="../../images/person-animation-plan/brave-work3-id63.png" width="490" alt="Brave work-three frames, ID 63"> |
| [ ] | Work 4 | 68 | <img src="../../images/person-animation-plan/brave-work4-id68.png" width="96" alt="Brave work-four frame, ID 68"> |
| [ ] | Work 5 | 73 | <img src="../../images/person-animation-plan/brave-work5-id73.png" width="372" alt="Brave work-five frames, ID 73"> |
| [ ] | Vehicle | 78 | <img src="../../images/person-animation-plan/brave-vehicle-id78.png" width="330" alt="Brave vehicle frames, ID 78"> |
| [ ] | Swim | 83 | <img src="../../images/person-animation-plan/brave-swim-id83.png" width="330" alt="Brave swim frames, ID 83"> |
| [ ] | Carry | 88 | <img src="../../images/person-animation-plan/brave-carry-id88.png" width="276" alt="Brave carry frames, ID 88"> |
| [ ] | Special | 100 | <img src="../../images/person-animation-plan/brave-special-id100.png" width="518" alt="Brave special frames, ID 100"> |
| [ ] | Ride | 110 | <img src="../../images/person-animation-plan/brave-ride-id110.png" width="330" alt="Brave ride frames, ID 110"> |
| [ ] | Dig / internal 1 | 115 | <img src="../../images/person-animation-plan/brave-dig-id115.png" width="512" alt="Brave dig frames, ID 115"> |
| [ ] | Build / internal 2 | 120 | <img src="../../images/person-animation-plan/brave-build-id120.png" width="350" alt="Brave build frames, ID 120"> |
| [ ] | Sit 1 | 131 | <img src="../../images/person-animation-plan/brave-sit1-id131.png" width="384" alt="Brave sit-one frames, ID 131"> |
| [ ] | Sit 2 | 136 | <img src="../../images/person-animation-plan/brave-sit2-id136.png" width="486" alt="Brave sit-two frames, ID 136"> |
| [ ] | Sit 3 | 141 | <img src="../../images/person-animation-plan/brave-sit3-id141.png" width="350" alt="Brave sit-three frames, ID 141"> |
| [ ] | Sit 4 | 146 | <img src="../../images/person-animation-plan/brave-sit4-id146.png" width="276" alt="Brave sit-four frames, ID 146"> |
| [ ] | Run | 156 | <img src="../../images/person-animation-plan/brave-run-id156.png" width="300" alt="Brave run frames, ID 156"> |

## Acceptance

- [ ] The renderer keeps subtype `2` through each state transition.
- [ ] The resolved VSTART and render type match the logical ID.
- [ ] The Rust frame count and order match the strip.
- [ ] An original-game capture and Rust capture agree on cadence and motion.
- [ ] Construction passes every check in the shared construction contract.
