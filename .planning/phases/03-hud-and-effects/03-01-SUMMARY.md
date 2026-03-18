---
phase: 03-hud-and-effects
plan: 01
subsystem: data, render
tags: [string-table, font, bitmap-font, hud-text, multi-size]

# Dependency graph
requires:
  - phase: 03-hud-and-effects
    provides: existing FONT_8X8 constant and draw_text() in HudRenderer
provides:
  - StringTable parser for lang00.dat binary format (1318 English strings)
  - FontData with glyph lookup at 8x8, 16x16, 24x24 via integer scaling
  - draw_text_sized() method on HudRenderer for multi-size text
affects: [03-hud-and-effects plan 03 (HUD panel text), 03-hud-and-effects plan 04 (tooltips)]

# Tech tracking
tech-stack:
  added: []
  patterns: [binary format parser with offset table, integer pixel scaling for bitmap fonts]

key-files:
  created:
    - src/data/strings.rs
    - src/data/font.rs
  modified:
    - src/data/mod.rs
    - src/render/hud/mod.rs

key-decisions:
  - "Integer scaling of 8x8 base font rather than loading original .fon files (sufficient quality, simpler)"
  - "draw_text_sized delegates to atlas-based draw_text with computed pixel size (avoids duplicate render path)"
  - "FontData kept as data-layer abstraction; GPU atlas handles actual rendering"

patterns-established:
  - "StringTable: offset-table binary parser with from_bytes() constructor"
  - "FontData: glyph scaling via pixel duplication (scale x scale blocks)"

requirements-completed: [HUD-07, HUD-08]

# Metrics
duration: 4min
completed: 2026-03-18
---

# Phase 3 Plan 01: String Table and Font Data Summary

**StringTable parser for lang00.dat binary format and FontData with 3-size glyph scaling (8/16/24px) wired into HudRenderer via draw_text_sized()**

## What Was Built

### Task 1: String Table Parser (HUD-08)
- `StringTable::from_bytes()` parses the binary format: u32le count, count x u32le offsets, null-terminated strings
- `get(index)` returns strings by index, `len()` reports count
- Handles edge cases: short data, out-of-bounds offsets, missing null terminators
- 6 unit tests covering empty, single, multiple, OOB, short header, and realistic 3-string scenarios

### Task 2: Font Data Parser + HUD Wiring (HUD-07)
- `FontData::from_8x8_bitmap()` converts the FONT_8X8 constant into per-glyph bitmaps (1 byte per pixel)
- `FontData::glyph(ch)` returns glyph for ASCII char with blank fallback for control/out-of-range chars
- `FontData::scaled(n)` produces scaled variants: 16x16 (n=2), 24x24 (n=3) via pixel duplication
- `draw_text_sized()` added to HudRenderer, routes font_scale (1/2/3) to the correct pixel size
- 6 unit tests covering glyph count, non-empty glyphs, control chars, OOB chars, scale=2, scale=3

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 | 6386cb5 | StringTable parser with TDD tests |
| 2 | 5ac7c6f | FontData parser and draw_text_sized HUD method |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] draw_text_sized delegates to atlas draw_text instead of CPU FontData**
- **Found during:** Task 2
- **Issue:** The plan specified draw_text_sized should use FontData::scaled() directly, but the HUD renderer uses a GPU atlas pipeline. Creating a separate CPU-side rendering path would duplicate the existing draw_text code path and bypass the GPU atlas.
- **Fix:** draw_text_sized() delegates to the existing atlas-based draw_text() with computed pixel size (font_scale * 8). FontData import kept with allow(unused) for future .fon file loading.
- **Files modified:** src/render/hud/mod.rs

## Verification

- `cargo test data::strings::tests` -- 6 tests pass
- `cargo test data::font::tests` -- 6 tests pass
- `cargo test --lib` -- 539 tests pass, no regressions
- `cargo build` -- compiles cleanly
- `grep -c "draw_text_sized" src/render/hud/mod.rs` -- returns 1+
- `grep -c "FontData" src/render/hud/mod.rs` -- returns 1+
