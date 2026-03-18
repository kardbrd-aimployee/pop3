---
phase: 02-economy-and-combat
plan: 10
subsystem: render
tags: [wgpu, ghost-preview, alpha-blending, shader-uniforms, building-mesh]

# Dependency graph
requires:
  - phase: 02-economy-and-combat
    provides: "GhostPreviewState in FrameState, building pipeline, ModelEnvelop"
provides:
  - "Ghost preview rendering with GPU uniforms and alpha blending"
  - "build_ghost_building_mesh() for single-building mesh at specific cell"
  - "ModelEnvelop::draw_single() for rendering individual models"
  - "GhostParams shader uniform struct (tint + alpha) on bind group 3"
affects: [03-spells-and-ai]

# Tech tracking
tech-stack:
  added: []
  patterns: [ghost-uniform-buffer, cached-ghost-mesh-key, alpha-blended-pipeline]

key-files:
  created: []
  modified:
    - src/render/buildings.rs
    - src/render/envelop.rs
    - src/render/app.rs
    - shaders/objects_tex.wgsl

key-decisions:
  - "Default tribe_index=0 (Blue) for ghost preview; TODO for player tribe selection"
  - "Ghost mesh cached by (building_type, cell_x, cell_y) tuple to avoid per-frame rebuild"
  - "Identity ghost uniforms [1,1,1,1] for normal building rendering (no visual regression)"
  - "Ghost pipeline depth_write_enabled=false so transparent ghost doesn't occlude objects behind"

patterns-established:
  - "Ghost uniform pattern: bind group 3 with identity defaults, overwritten per-frame for ghost draw"
  - "Cached mesh rebuild: compare key tuple, only rebuild on change"

requirements-completed: [BLDG-03]

# Metrics
duration: 5min
completed: 2026-03-18
---

# Phase 2 Plan 10: Ghost Preview Rendering Summary

**Semi-transparent ghost building mesh with GPU uniforms, alpha blending, green/red tint, and per-position cached mesh rebuild**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-18T04:27:34Z
- **Completed:** 2026-03-18T04:32:42Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Ghost preview renders as semi-transparent building mesh at the ghost cell position
- Green tint [0.3, 1.0, 0.3] for valid placement, red tint [1.0, 0.3, 0.3] for invalid
- Normal buildings render identically to before (ghost uniforms set to identity)
- Ghost mesh cached and only rebuilt when building_type or cell position changes

## Task Commits

Each task was committed atomically:

1. **Task 1: Add build_ghost_building_mesh and draw_single to ModelEnvelop** - `a8c81a2` (feat)
2. **Task 2: Add ghost shader uniforms, pipeline, buffer, and draw call** - `53fd526` (feat)

## Files Created/Modified
- `src/render/buildings.rs` - Added build_ghost_building_mesh() for single-building ghost mesh
- `src/render/envelop.rs` - Added draw_single() and len() to ModelEnvelop
- `src/render/app.rs` - Ghost uniform buffer, bind group, pipeline, cached mesh rebuild, draw call
- `shaders/objects_tex.wgsl` - GhostParams struct on bind group 3, tint/alpha in fragment output

## Decisions Made
- Default tribe_index=0 (Blue) for ghost preview until player tribe tracking is wired
- Ghost mesh cached by (building_type, cell_x, cell_y) key to avoid per-frame GPU buffer creation
- Identity ghost uniforms [1,1,1,1] ensure no visual regression for normal buildings
- Ghost pipeline uses depth_write_enabled=false to prevent transparent ghost from occluding objects

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Ghost preview rendering complete, ready for integration with building placement UI
- Player tribe selection for ghost mesh is a minor follow-up (defaults to Blue tribe)

---
*Phase: 02-economy-and-combat*
*Completed: 2026-03-18*
