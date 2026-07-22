# Preacher animation checklist

Native subtype: `4`

Primary mechanics: training, movement, preaching, conversion pressure, combat, vehicles, and water

Extracted original-game sequences: `23`

Shared rules: [person state and animation checklist](../person-state-animation-checklist.md)

## Current Rust state adapter

| Check | Exact `PersonState` values | Row and preacher ID | Verification |
|---|---|---|---|
| [ ] | `Idle`, `InsideTraining`, `InShield`, `WaitingAtReincPillar` | Idle row 0, ID 17 | Capture open |
| [ ] | `Moving`, `Wander`, `GoToPoint`, `FollowPath`, `GoToMarker`, `WaitForPath`, `WaitAtMarker`, `EnterBuilding`, `WaitOutside`, `Training`, `Housing`, `Gathering`, `Spawning`, `BeingConverted`, `WaitingAfterConvert`, `WaitingForBoat`, `Placeholder`, `GetOffBoat`, `EnteringVehicle`, `Teleporting`, `InternalState`, `InShieldIdle` | Walk row 1, ID 23; zero speed falls back to ID 17 | Mixed verified and provisional mappings |
| [ ] | `InsideBuilding`, `InTraining`, `Fighting` | Action row 3, ID 34 | Handler overrides open |
| [ ] | `Dying`, `Dead`, `BeingSacrificed` | Die row 6, ID 29 | Sacrifice mapping open |
| [ ] | `Celebrating` | Celebrate row 7, ID 40 | Capture open |
| [ ] | `GatheringWood` | Work row 13, ID 75 | Mechanic assignment open |
| [ ] | `Drowning`, `WaitingInWater` | Swim row 16, ID 85 | Waterline capture open |
| [ ] | `CarryingWood` | Carry row 18, ID 90 | Mechanic assignment open |
| [ ] | `Building` | Walk row 1, ID 23 | Preachers must not receive brave construction jobs |
| [ ] | `SitDown` | Sit row 21, ID 133 | Three other variants remain unselected |
| [ ] | `Fleeing`, `Preaching`, `ExitingVehicle` | Run row 25, ID 158 | Preaching target action open |

## State mapping

| Check | States or mechanic | Planned sequence | Status |
|---|---|---|---|
| [ ] | Idle-class states | Idle row 0, ID 17<br><img src="../../images/person-animation-plan/preacher-idle-id17.png" width="276" alt="Preacher idle frames, ID 17"> | Cadence capture open |
| [ ] | Moving, path, marker, and entrance travel | Walk row 1, ID 23<br><img src="../../images/person-animation-plan/preacher-walk-id23.png" width="288" alt="Preacher walk frames, ID 23"> | Runtime mapping exists |
| [ ] | Preaching target travel | Run row 25, ID 158<br><img src="../../images/person-animation-plan/preacher-run-id158.png" width="300" alt="Preacher run frames, ID 158"> | Native entry mapping exists |
| [ ] | Preaching at target | Action or work row, unassigned<br><img src="../../images/person-animation-plan/preacher-action-id34.png" width="360" alt="Preacher action frames, ID 34"> | Original target-action capture required |
| [ ] | Fighting and training actions | Action row 3, ID 34<br><img src="../../images/person-animation-plan/preacher-action-id34.png" width="360" alt="Preacher action frames, ID 34"> | Attack and preaching actions need separate evidence |
| [ ] | Dying and dead hold | Die row 6, ID 29<br><img src="../../images/person-animation-plan/preacher-die-id29.png" width="324" alt="Preacher death frames, ID 29"> | One-shot and final-frame rules open |
| [ ] | Drowning and waiting in water | Swim row 16, ID 85<br><img src="../../images/person-animation-plan/preacher-swim-id85.png" width="330" alt="Preacher swim frames, ID 85"> | Waterline offset open |
| [ ] | SitDown | IDs 133, 138, 143, and 148 | Variant selector open |
| [ ] | Vehicle entry, travel, and exit | Walk, vehicle ID 80, ride ID 112, then run | Transition capture open |
| [ ] | Carry, dig, build, and work rows | Extracted but unassigned | Do not inherit brave construction rules |
| [ ] | Conversion reaction, spawning, sacrifice, teleport, and internal states | Unassigned | Handler evidence required |

## Extracted sequence inventory

| Check | Native row or sequence | Logical ID | Original frames |
|---|---|---:|---|
| [ ] | Idle | 17 | <img src="../../images/person-animation-plan/preacher-idle-id17.png" width="276" alt="Preacher idle frames, ID 17"> |
| [ ] | Walk | 23 | <img src="../../images/person-animation-plan/preacher-walk-id23.png" width="288" alt="Preacher walk frames, ID 23"> |
| [ ] | Die | 29 | <img src="../../images/person-animation-plan/preacher-die-id29.png" width="324" alt="Preacher death frames, ID 29"> |
| [ ] | Action | 34 | <img src="../../images/person-animation-plan/preacher-action-id34.png" width="360" alt="Preacher action frames, ID 34"> |
| [ ] | Celebrate | 40 | <img src="../../images/person-animation-plan/preacher-celebrate-id40.png" width="330" alt="Preacher celebrate frames, ID 40"> |
| [ ] | Spell idle | 45 | <img src="../../images/person-animation-plan/preacher-spell-idle-id45.png" width="88" alt="Preacher spell-idle frame, ID 45"> |
| [ ] | Spell walk | 50 | <img src="../../images/person-animation-plan/preacher-spell-walk-id50.png" width="300" alt="Preacher spell-walk frames, ID 50"> |
| [ ] | Work 1 | 55 | <img src="../../images/person-animation-plan/preacher-work1-id55.png" width="504" alt="Preacher work-one frames, ID 55"> |
| [ ] | Work 2 | 60 | <img src="../../images/person-animation-plan/preacher-work2-id60.png" width="378" alt="Preacher work-two frames, ID 60"> |
| [ ] | Work 3 | 65 | <img src="../../images/person-animation-plan/preacher-work3-id65.png" width="490" alt="Preacher work-three frames, ID 65"> |
| [ ] | Work 4 | 70 | <img src="../../images/person-animation-plan/preacher-work4-id70.png" width="96" alt="Preacher work-four frame, ID 70"> |
| [ ] | Work 5 | 75 | <img src="../../images/person-animation-plan/preacher-work5-id75.png" width="372" alt="Preacher work-five frames, ID 75"> |
| [ ] | Vehicle | 80 | <img src="../../images/person-animation-plan/preacher-vehicle-id80.png" width="330" alt="Preacher vehicle frames, ID 80"> |
| [ ] | Swim | 85 | <img src="../../images/person-animation-plan/preacher-swim-id85.png" width="330" alt="Preacher swim frames, ID 85"> |
| [ ] | Carry | 90 | <img src="../../images/person-animation-plan/preacher-carry-id90.png" width="276" alt="Preacher carry frames, ID 90"> |
| [ ] | Ride | 112 | <img src="../../images/person-animation-plan/preacher-ride-id112.png" width="340" alt="Preacher ride frames, ID 112"> |
| [ ] | Dig / internal 1 | 117 | <img src="../../images/person-animation-plan/preacher-dig-id117.png" width="512" alt="Preacher dig frames, ID 117"> |
| [ ] | Build / internal 2 | 122 | <img src="../../images/person-animation-plan/preacher-build-id122.png" width="350" alt="Preacher build frames, ID 122"> |
| [ ] | Sit 1 | 133 | <img src="../../images/person-animation-plan/preacher-sit1-id133.png" width="384" alt="Preacher sit-one frames, ID 133"> |
| [ ] | Sit 2 | 138 | <img src="../../images/person-animation-plan/preacher-sit2-id138.png" width="486" alt="Preacher sit-two frames, ID 138"> |
| [ ] | Sit 3 | 143 | <img src="../../images/person-animation-plan/preacher-sit3-id143.png" width="350" alt="Preacher sit-three frames, ID 143"> |
| [ ] | Sit 4 | 148 | <img src="../../images/person-animation-plan/preacher-sit4-id148.png" width="276" alt="Preacher sit-four frames, ID 148"> |
| [ ] | Run | 158 | <img src="../../images/person-animation-plan/preacher-run-id158.png" width="300" alt="Preacher run frames, ID 158"> |

## Acceptance

- [ ] The renderer keeps subtype `4` through each state transition.
- [ ] The resolved VSTART and render type match the logical ID.
- [ ] The Rust frame count and order match the strip.
- [ ] Training produces subtype `4` at the building entrance.
- [ ] Preaching travel and target action use separate verified sequences.
