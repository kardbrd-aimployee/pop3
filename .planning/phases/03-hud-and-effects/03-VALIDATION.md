---
phase: 3
slug: hud-and-effects
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-18
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `cargo test` |
| **Config file** | Cargo.toml (standard) |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 03-01-01 | 01 | 1 | HUD-01 | unit | `cargo test hud::tests::minimap` | Partial | ⬜ pending |
| 03-01-02 | 01 | 1 | HUD-03 | unit | `cargo test hud::tests::mana_bar` | ❌ W0 | ⬜ pending |
| 03-01-03 | 01 | 1 | HUD-04 | unit | `cargo test hud::tests::population` | ❌ W0 | ⬜ pending |
| 03-02-01 | 02 | 1 | HUD-07 | unit | `cargo test data::font` | ❌ W0 | ⬜ pending |
| 03-02-02 | 02 | 1 | HUD-08 | unit | `cargo test data::strings` | ❌ W0 | ⬜ pending |
| 03-03-01 | 03 | 1 | FX-01 | unit | `cargo test engine::effects::pool` | ❌ W0 | ⬜ pending |
| 03-03-02 | 03 | 1 | FX-02 | unit | `cargo test engine::effects::spell` | ❌ W0 | ⬜ pending |
| 03-03-03 | 03 | 1 | FX-03 | unit | `cargo test engine::effects::combat` | ❌ W0 | ⬜ pending |
| 03-03-04 | 03 | 1 | FX-04 | unit | `cargo test engine::effects::building` | ❌ W0 | ⬜ pending |
| 03-03-05 | 03 | 1 | FX-05 | unit | `cargo test engine::effects::attach` | ❌ W0 | ⬜ pending |
| 03-04-01 | 04 | 2 | HUD-02 | unit | `cargo test hud::tests::spell_bar` | ❌ W0 | ⬜ pending |
| 03-04-02 | 04 | 2 | HUD-05 | unit | `cargo test hud::tests::info_panel` | ❌ W0 | ⬜ pending |
| 03-04-03 | 04 | 2 | HUD-06 | unit | `cargo test hud::tests::health_bar` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/engine/effects/mod.rs` — Effect struct, EffectPool, EffectType enum, basic alloc/free tests
- [ ] `src/data/strings.rs` — String table parser with tests
- [ ] Extend `src/render/hud/mod.rs` tests for new HudState fields (mana bar, spell cooldowns)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Minimap visual correctness | HUD-01 | GPU rendering | Run game, verify minimap shows terrain + colored dots |
| Health bars above units | HUD-06 | GPU rendering | Damage a unit, verify bar appears above it |
| Effect visual rendering | FX-02..04 | GPU rendering | Trigger combat/spell, verify particles appear |
| Font rendering at 3 sizes | HUD-07 | GPU rendering | Check 12/16/24pt text renders correctly |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
