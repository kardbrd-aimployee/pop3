---
phase: 03-hud-and-effects
plan: 04
subsystem: ui
tags: [minimap, viewport, selection, hud, toroidal-wrapping]

# Dependency graph
requires:
  - phase: 03-hud-and-effects/03
    provides: "HudState with mana, population, spell cooldowns"
provides:
  - "MinimapViewport struct for camera position overlay on minimap"
  - "Click-to-move on minimap with toroidal wrapping"
  - "SelectedEntityInfo struct for unit detail panel in sidebar"
  - "unit_subtype_name helper mapping subtype IDs to display names"
affects: [04-ai-and-scripting]

# Tech tracking
tech-stack:
  added: []
  patterns: [toroidal-delta-wrapping, minimap-coordinate-mapping]

key-files:
  created: []
  modified:
    - src/render/hud/mod.rs
    - src/render/app.rs

key-decisions:
  - "Viewport rect size derived from zoom level: 20.0/zoom cells wide, aspect-ratio-corrected height"
  - "Camera center from get_shift_vector() rem_euclid 128 for toroidal cell coords"
  - "Minimap click uses rebuild_spawn_model() same as keyboard panning for consistency"

patterns-established:
  - "Minimap coordinate helpers (minimap_click_to_cell, toroidal_delta) as pure functions in hud/mod.rs"
  - "Selection info populated from first selected unit with fallback to None"

requirements-completed: [HUD-01, HUD-05]

# Metrics
duration: 4min
completed: 2026-03-18
---

# Phase 3 Plan 4: Minimap Viewport and Selection Info Summary

**Minimap viewport rectangle with click-to-move navigation and sidebar selection info panel with color-coded health bar**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-18T15:29:27Z
- **Completed:** 2026-03-18T15:33:20Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- White viewport rectangle on minimap tracking camera position with zoom-dependent size
- Click-to-move on minimap with toroidal wrapping (shortest path across map edges)
- Selection info panel showing unit name, color-coded HP bar, state, and multi-select count
- 12 unit tests for minimap_click_to_cell, toroidal_delta, and unit_subtype_name helpers

## Task Commits

Each task was committed atomically:

1. **Task 1+2: Minimap viewport, click-to-move, selection info panel** - `4aa16c6` (feat)

**Plan metadata:** [pending] (docs: complete plan)

## Files Created/Modified
- `src/render/hud/mod.rs` - MinimapViewport, SelectedEntityInfo structs, minimap_click_to_cell, toroidal_delta, unit_subtype_name helpers, tests
- `src/render/app.rs` - camera_viewport computation in build_hud_state, viewport rect rendering, minimap click handler, selection info panel rendering

## Decisions Made
- Viewport width = 20.0 / zoom cells; height derived from screen aspect ratio
- Camera center extracted from get_shift_vector().rem_euclid(128.0) matching toroidal map
- Minimap click triggers rebuild_spawn_model() consistent with keyboard panning code path
- Combined Task 1 and Task 2 into single commit since changes are deeply interleaved in same files

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed rebuild_terrain reference to rebuild_spawn_model**
- **Found during:** Task 1 (minimap click handler)
- **Issue:** Plan suggested `rebuild_terrain` field which does not exist on App struct
- **Fix:** Used `rebuild_spawn_model()` method matching existing keyboard panning pattern
- **Files modified:** src/render/app.rs
- **Verification:** cargo build succeeds
- **Committed in:** 4aa16c6

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Minor reference correction, no scope change.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Minimap viewport and selection panel complete, ready for Phase 3 Plan 5
- All 550 library tests passing with no regressions

---
*Phase: 03-hud-and-effects*
*Completed: 2026-03-18*
