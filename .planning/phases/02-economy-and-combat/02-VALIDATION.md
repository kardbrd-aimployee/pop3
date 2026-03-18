---
phase: 2
slug: economy-and-combat
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-17
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `cargo test` |
| **Config file** | Cargo.toml (standard) |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 01 | 1 | BLDG-01 | unit | `cargo test buildings::state_machine` | ❌ W0 | ⬜ pending |
| 02-01-02 | 01 | 1 | BLDG-02 | unit | `cargo test buildings::tick::construction` | ❌ W0 | ⬜ pending |
| 02-01-03 | 01 | 1 | BLDG-03 | unit | `cargo test buildings::placement` | ❌ W0 | ⬜ pending |
| 02-01-04 | 01 | 1 | BLDG-04 | unit | `cargo test buildings::occupants` | ❌ W0 | ⬜ pending |
| 02-01-05 | 01 | 1 | BLDG-05 | unit | `cargo test buildings::spawning` | ❌ W0 | ⬜ pending |
| 02-01-06 | 01 | 1 | BLDG-06 | unit | `cargo test buildings::training` | ❌ W0 | ⬜ pending |
| 02-01-07 | 01 | 1 | BLDG-07 | unit | `cargo test buildings::damage` | ❌ W0 | ⬜ pending |
| 02-01-08 | 01 | 1 | BLDG-08 | unit | `cargo test buildings::combat` | ❌ W0 | ⬜ pending |
| 02-02-01 | 02 | 1 | ECON-01 | unit | `cargo test economy::wood` | ❌ W0 | ⬜ pending |
| 02-02-02 | 02 | 1 | ECON-02 | unit | `cargo test economy::wood::storage` | ❌ W0 | ⬜ pending |
| 02-02-03 | 02 | 1 | ECON-03 | unit | `cargo test economy::mana` | ❌ W0 | ⬜ pending |
| 02-02-04 | 02 | 1 | ECON-04 | unit | `cargo test economy::mana::pool` | ❌ W0 | ⬜ pending |
| 02-02-05 | 02 | 1 | ECON-05 | unit | `cargo test economy::population` | ❌ W0 | ⬜ pending |
| 02-03-01 | 03 | 1 | PRSN-01 | unit | `cargo test person_state::enter_building` | ❌ W0 | ⬜ pending |
| 02-03-02 | 03 | 1 | PRSN-02 | unit | `cargo test person_state::exit_building` | ❌ W0 | ⬜ pending |
| 02-03-03 | 03 | 1 | PRSN-03 | unit | `cargo test person_state::housed` | ❌ W0 | ⬜ pending |
| 02-03-04 | 03 | 1 | PRSN-04 | unit | `cargo test person_state::training` | ❌ W0 | ⬜ pending |
| 02-03-05 | 03 | 1 | PRSN-05 | unit | `cargo test person_state::gather_wood` | ❌ W0 | ⬜ pending |
| 02-03-06 | 03 | 1 | PRSN-06 | unit | `cargo test person_state::tick_drowning` | ✅ | ⬜ pending |
| 02-03-07 | 03 | 1 | PRSN-07 | unit | `cargo test person_state::guard` | ❌ W0 | ⬜ pending |
| 02-03-08 | 03 | 1 | PRSN-08 | unit | `cargo test person_state::death_cleanup` | ❌ W0 | ⬜ pending |
| 02-04-01 | 04 | 2 | CMBT-01 | unit | `cargo test person_state::calculate_melee_damage` | ✅ | ⬜ pending |
| 02-04-02 | 04 | 2 | CMBT-02 | unit | `cargo test combat::projectile` | ❌ W0 | ⬜ pending |
| 02-04-03 | 04 | 2 | CMBT-03 | unit | `cargo test combat::drum_tower` | ❌ W0 | ⬜ pending |
| 02-04-04 | 04 | 2 | CMBT-04 | unit | `cargo test combat::death_cleanup` | ❌ W0 | ⬜ pending |
| 02-04-05 | 04 | 2 | CMBT-05 | unit | `cargo test combat::knockback` | ❌ W0 | ⬜ pending |
| 02-05-01 | 05 | 1 | TERR-01 | unit | `cargo test terrain::modify_height` | ❌ W0 | ⬜ pending |
| 02-05-02 | 05 | 1 | TERR-02 | unit | `cargo test terrain::cascade` | ❌ W0 | ⬜ pending |
| 02-05-03 | 05 | 1 | TERR-03 | unit | `cargo test terrain::water_interaction` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/engine/buildings/` module -- entire new module tree with test stubs
- [ ] `src/engine/economy/` module -- entire new module tree with test stubs
- [ ] `src/engine/combat/` module -- projectile/knockback tests (melee damage test exists)
- [ ] `src/engine/terrain/` module -- modification and cascade tests
- [ ] Person state test extensions for new states (EnterBuilding, Housed, Training, GatherWood, Guard)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Ghost building preview rendering | BLDG-03 | Visual/GPU rendering | Place building, verify transparent preview follows cursor |
| Construction animation | BLDG-02 | Visual animation timing | Start construction, watch scaling/building visual |
| Combat visual feedback | CMBT-01 | Visual/particle effects | Start melee fight, verify hit effects render |
| Terrain mesh deformation | TERR-01 | Visual mesh update | Raise/lower terrain, verify mesh renders correctly |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
