# Firewarrior animation checklist

Native subtype: `6`

Primary mechanics: training, movement, ranged fire attacks, melee combat, vehicles, and water

Extracted original-game sequences: `24`

Shared rules: [person state and animation checklist](../person-state-animation-checklist.md)

## Current Rust state adapter

| Check | Exact `PersonState` values | Row and firewarrior ID | Verification |
|---|---|---|---|
| [ ] | `Idle`, `InsideTraining`, `InShield`, `WaitingAtReincPillar` | Idle row 0, ID 19 | Capture open |
| [ ] | `Moving`, `Wander`, `GoToPoint`, `FollowPath`, `GoToMarker`, `WaitForPath`, `WaitAtMarker`, `EnterBuilding`, `WaitOutside`, `Training`, `Housing`, `Gathering`, `Spawning`, `BeingConverted`, `WaitingAfterConvert`, `WaitingForBoat`, `Placeholder`, `GetOffBoat`, `EnteringVehicle`, `Teleporting`, `InternalState`, `InShieldIdle` | Walk row 1, ID 25; zero speed falls back to ID 19 | Mixed verified and provisional mappings |
| [ ] | `InsideBuilding`, `InTraining`, `Fighting` | Action row 3, ID 36 | Handler overrides open |
| [ ] | `Dying`, `Dead`, `BeingSacrificed` | Die row 6, ID 31 | Sacrifice mapping open |
| [ ] | `Celebrating` | Celebrate row 7, ID 42 | Capture open |
| [ ] | `GatheringWood` | Work row 13, ID 77 | Mechanic assignment open |
| [ ] | `Drowning`, `WaitingInWater` | Swim row 16, ID 87 | Waterline capture open |
| [ ] | `CarryingWood` | Carry row 18, ID 92 | Mechanic assignment open |
| [ ] | `Building` | Walk row 1, ID 25 | Firewarriors must not receive brave construction jobs |
| [ ] | `SitDown` | Sit row 21, ID 135 | Three other variants remain unselected |
| [ ] | `Fleeing`, `Preaching`, `ExitingVehicle` | Run row 25, ID 160 | Handler use open |

## State mapping

| Check | States or mechanic | Planned sequence | Status |
|---|---|---|---|
| [ ] | Idle-class states | Idle row 0, ID 19<br><img src="../../images/person-animation-plan/firewarrior-idle-id19.png" width="300" alt="Firewarrior idle frames, ID 19"> | Cadence capture open |
| [ ] | Moving, path, marker, and entrance travel | Walk row 1, ID 25<br><img src="../../images/person-animation-plan/firewarrior-walk-id25.png" width="288" alt="Firewarrior walk frames, ID 25"> | Runtime mapping exists |
| [ ] | Fighting and training actions | Action row 3, ID 36<br><img src="../../images/person-animation-plan/firewarrior-action-id36.png" width="372" alt="Firewarrior action frames, ID 36"> | Attack timing open |
| [ ] | Ranged fire attack | Special candidate, ID 101<br><img src="../../images/person-animation-plan/firewarrior-special-id101.png" width="518" alt="Firewarrior special frames, ID 101"> | Extracted body and flame layers match firewarrior; native table ownership needs audit |
| [ ] | Dying and dead hold | Die row 6, ID 31<br><img src="../../images/person-animation-plan/firewarrior-die-id31.png" width="360" alt="Firewarrior death frames, ID 31"> | One-shot and final-frame rules open |
| [ ] | Fleeing and fast exit | Run row 25, ID 160<br><img src="../../images/person-animation-plan/firewarrior-run-id160.png" width="312" alt="Firewarrior run frames, ID 160"> | Exit capture open |
| [ ] | Drowning and waiting in water | Swim row 16, ID 87<br><img src="../../images/person-animation-plan/firewarrior-swim-id87.png" width="330" alt="Firewarrior swim frames, ID 87"> | Waterline offset open |
| [ ] | SitDown | IDs 135, 140, 145, and 150 | Variant selector open |
| [ ] | Vehicle entry, travel, and exit | Walk, vehicle ID 82, ride ID 114, then run | Transition capture open |
| [ ] | Carry, dig, build, and work rows | Extracted but unassigned | Do not inherit brave construction rules |
| [ ] | Spawning, sacrifice, conversion, teleport, and internal states | Unassigned | Handler evidence required |

## Extracted sequence inventory

| Check | Native row or sequence | Logical ID | Original frames |
|---|---|---:|---|
| [ ] | Idle | 19 | <img src="../../images/person-animation-plan/firewarrior-idle-id19.png" width="300" alt="Firewarrior idle frames, ID 19"> |
| [ ] | Walk | 25 | <img src="../../images/person-animation-plan/firewarrior-walk-id25.png" width="288" alt="Firewarrior walk frames, ID 25"> |
| [ ] | Die | 31 | <img src="../../images/person-animation-plan/firewarrior-die-id31.png" width="360" alt="Firewarrior death frames, ID 31"> |
| [ ] | Action | 36 | <img src="../../images/person-animation-plan/firewarrior-action-id36.png" width="372" alt="Firewarrior action frames, ID 36"> |
| [ ] | Celebrate | 42 | <img src="../../images/person-animation-plan/firewarrior-celebrate-id42.png" width="360" alt="Firewarrior celebrate frames, ID 42"> |
| [ ] | Spell idle | 47 | <img src="../../images/person-animation-plan/firewarrior-spell-idle-id47.png" width="88" alt="Firewarrior spell-idle frame, ID 47"> |
| [ ] | Spell walk | 52 | <img src="../../images/person-animation-plan/firewarrior-spell-walk-id52.png" width="300" alt="Firewarrior spell-walk frames, ID 52"> |
| [ ] | Work 1 | 57 | <img src="../../images/person-animation-plan/firewarrior-work1-id57.png" width="546" alt="Firewarrior work-one frames, ID 57"> |
| [ ] | Work 2 | 62 | <img src="../../images/person-animation-plan/firewarrior-work2-id62.png" width="434" alt="Firewarrior work-two frames, ID 62"> |
| [ ] | Work 3 | 67 | <img src="../../images/person-animation-plan/firewarrior-work3-id67.png" width="546" alt="Firewarrior work-three frames, ID 67"> |
| [ ] | Work 4 | 72 | <img src="../../images/person-animation-plan/firewarrior-work4-id72.png" width="112" alt="Firewarrior work-four frame, ID 72"> |
| [ ] | Work 5 | 77 | <img src="../../images/person-animation-plan/firewarrior-work5-id77.png" width="372" alt="Firewarrior work-five frames, ID 77"> |
| [ ] | Vehicle | 82 | <img src="../../images/person-animation-plan/firewarrior-vehicle-id82.png" width="340" alt="Firewarrior vehicle frames, ID 82"> |
| [ ] | Swim | 87 | <img src="../../images/person-animation-plan/firewarrior-swim-id87.png" width="330" alt="Firewarrior swim frames, ID 87"> |
| [ ] | Carry | 92 | <img src="../../images/person-animation-plan/firewarrior-carry-id92.png" width="276" alt="Firewarrior carry frames, ID 92"> |
| [ ] | Special | 101 | <img src="../../images/person-animation-plan/firewarrior-special-id101.png" width="518" alt="Firewarrior special frames, ID 101"> |
| [ ] | Ride | 114 | <img src="../../images/person-animation-plan/firewarrior-ride-id114.png" width="340" alt="Firewarrior ride frames, ID 114"> |
| [ ] | Dig / internal 1 | 119 | <img src="../../images/person-animation-plan/firewarrior-dig-id119.png" width="512" alt="Firewarrior dig frames, ID 119"> |
| [ ] | Build / internal 2 | 124 | <img src="../../images/person-animation-plan/firewarrior-build-id124.png" width="350" alt="Firewarrior build frames, ID 124"> |
| [ ] | Sit 1 | 135 | <img src="../../images/person-animation-plan/firewarrior-sit1-id135.png" width="384" alt="Firewarrior sit-one frames, ID 135"> |
| [ ] | Sit 2 | 140 | <img src="../../images/person-animation-plan/firewarrior-sit2-id140.png" width="486" alt="Firewarrior sit-two frames, ID 140"> |
| [ ] | Sit 3 | 145 | <img src="../../images/person-animation-plan/firewarrior-sit3-id145.png" width="392" alt="Firewarrior sit-three frames, ID 145"> |
| [ ] | Sit 4 | 150 | <img src="../../images/person-animation-plan/firewarrior-sit4-id150.png" width="276" alt="Firewarrior sit-four frames, ID 150"> |
| [ ] | Run | 160 | <img src="../../images/person-animation-plan/firewarrior-run-id160.png" width="312" alt="Firewarrior run frames, ID 160"> |

## Acceptance

- [ ] The renderer keeps subtype `6` through each state transition.
- [ ] The resolved VSTART and render type match the logical ID.
- [ ] The Rust frame count and order match the strip.
- [ ] Training produces subtype `6` at the building entrance.
- [ ] The ranged attack synchronizes the flame projectile with its release frame.
- [ ] A binary audit resolves logical ID 101 ownership.
