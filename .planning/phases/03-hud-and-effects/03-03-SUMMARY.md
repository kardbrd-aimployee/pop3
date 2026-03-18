---
phase: 03-hud-and-effects
plan: 03
subsystem: render
tags: [hud, mana-bar, population, spell-cooldown, data-contract]

# Dependency graph
requires:
  - phase: 02-economy-and-combat
    provides: TribeData with mana, population, max_population fields
provides:
  - Extended HudState with player_mana, player_max_mana, player_population, player_max_population, spell_cooldowns
  - SpellCooldown struct for cooldown overlay infrastructure
  - compute_mana_fraction helper function
  - Mana bar rendering (proportional blue fill with label)
  - Population display rendering ("Pop: X/Y")
  - Spell cooldown overlay rendering (darkened proportional to remaining)
affects: [04-spell-system (spell_cooldowns population), 03-hud-and-effects plan 05 (health bars)]

# Tech tracking
tech-stack:
  added: []
  patterns: [HudState data contract extension, compute helper with unit tests]

key-files:
  created: []
  modified:
    - src/render/hud/mod.rs
    - src/render/app.rs

key-decisions:
  - "Mana displayed in K units (player_mana / 1000) for readability"
  - "spell_cooldowns as Vec<SpellCooldown> populated empty now, Phase 4 fills from SpellSystem"
  - "Population display placed below mana bar in sidebar layout"

patterns-established:
  - "compute_mana_fraction: pure helper with edge-case tests (zero max, overflow clamping)"
  - "Cooldown overlay: draw darkened rect before text, proportional to remaining/total"

metrics:
  duration: ~4 min
  completed: 2026-03-18
  tasks: 2
  files: 2
---

# Phase 3 Plan 3: HudState Extensions (Mana, Population, Spell Cooldowns) Summary

Extended HudState data contract with mana bar, population display, and spell cooldown overlay infrastructure, all rendering in the sidebar below the minimap.

## What Was Done

### Task 1: Extend HudState with mana, population, and spell cooldown fields
- Added `SpellCooldown` struct (spell_index, cooldown_remaining, cooldown_total)
- Extended `HudState` with `player_mana`, `player_max_mana`, `player_population`, `player_max_population`, `spell_cooldowns`
- Added `compute_mana_fraction(mana, max_mana) -> f32` helper clamped to [0.0, 1.0]
- Updated `build_hud_state()` to populate from `game_world.tribes.tribes[0]` (player tribe)
- Commit: `b8de761`

### Task 2: Render mana bar, population display, and spell cooldown overlays
- Mana bar: blue proportional fill below minimap with dark background, "Mana: XK" label
- Population display: "Pop: X/Y" text below mana bar in green
- Spell cooldown overlay: darkened rect over spell entries proportional to cooldown_remaining/cooldown_total
- 5 unit tests: mana fraction (zero max, full, half, overflow clamp) and SpellCooldown struct
- Commit: `0a07987`

## Deviations from Plan

None -- plan executed exactly as written.

## Verification

- `cargo build`: compiles cleanly (warnings only, pre-existing)
- `cargo test hud`: 31 tests pass (5 new)
- `cargo test --lib`: 539 tests pass (no regressions)

## Commits

| Task | Commit | Message |
|------|--------|---------|
| 1 | b8de761 | feat(03-03): extend HudState with mana, population, and spell cooldown fields |
| 2 | 0a07987 | feat(03-03): render mana bar, population display, and spell cooldown overlays |
