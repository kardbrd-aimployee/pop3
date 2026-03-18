# Phase 3: HUD and Effects - Research

**Researched:** 2026-03-18
**Domain:** HUD rendering, visual effect system, font/string loading, minimap interaction
**Confidence:** HIGH

## Summary

Phase 3 extends the existing HUD infrastructure (HudRenderer with GPU pipeline, font atlas, minimap texture, tab system, sprite atlas) with interactive elements (spell buttons, mana/population bars, health bars, info panels, minimap click-to-move) and builds a 512-slot visual effect pool with a state-machine-driven update loop. The existing codebase already has a solid `HudState` data contract pattern, a working HudRenderer with `draw_rect`, `draw_text`, `draw_sprite`, and minimap rendering -- this phase fills in the missing interactive and informational HUD elements and adds the effect system from scratch.

The codebase uses wgpu 28.0, Rust 2021 edition, and a clean separation between game state (engine/) and rendering (render/). The HUD follows a "data contract" pattern where `build_hud_state()` in app.rs produces a `HudState` struct consumed by the renderer -- all new HUD features should extend this pattern. The effect system is a new module under `engine/effects/` with a 512-slot pool, 64-byte Effect struct, and per-type state machine handlers.

**Primary recommendation:** Extend `HudState` with mana/population/spell/selection/health data, add corresponding rendering in `draw_hud()`, then build the effect pool as a new `engine/effects/` module with rendering integration in the world pass.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| HUD-01 | Minimap rendering (128x128, tribe-colored unit dots) | Minimap texture generation + dot rendering already exist in HudRenderer. Need: camera viewport rectangle overlay, click-to-move camera support via minimap coordinate mapping. |
| HUD-02 | Spell bar with available spells and cooldown indicators | Existing panel_entries for Spells tab lists 16 spells. Need: cooldown timer data in HudState, visual cooldown overlay (darkened rect + timer text), spell availability from tribe data. |
| HUD-03 | Mana bar display | TribeData.mana (u32, max 1,000,000) exists. Need: mana bar rect in HudState, proportional fill rendering. |
| HUD-04 | Population display | TribeData.population and max_population exist. Need: pop count text in HudState, bar or numeric display. |
| HUD-05 | Unit/building info panel on selection | SelectionState.selected exists with Vec<UnitId>. Need: selected entity data extraction (name, health, subtype, stats), rendering in sidebar panel area. |
| HUD-06 | Health bars above units and buildings | Unit.health/max_health fields exist. Need: world-to-screen projection for health bar positioning, HudState health bar entries, rendering in HUD pass after world. Original uses depth-bucketed sprites (type 0x0F-0x12). |
| HUD-07 | Font loading and text rendering (12/16/24pt) | Current: 8x8 bitmap font with draw_text(). Original: bitmap fonts at 12/16/24pt (2-3 bytes/row). For v1: extend current approach or load original font data. Original font files: font12j.fon, font16j.fon, font24j.fon. |
| HUD-08 | String table loading (English, 0x526 strings) | LANGUAGE/lang00.dat contains 1318 null-terminated strings. Need: binary parser for string table format (header: string count + offsets, then null-terminated strings). |
| FX-01 | Effect pool (512 max, 64 bytes per effect) | No effect code exists. Need: new module with Effect struct (type, state, flags, owner, position, velocity, frame, scale, alpha, target, damage, radius, duration, color, linked-list pointers), pre-allocated pool with free list. |
| FX-02 | Spell impact visual effects (burn, blast, lightning) | Types 0x01-0x1F in spec. Need: per-type init + update handlers, particle sprite rendering. Depends on sprite bank loading for effect sprites (0x3a-0x3d etc.). |
| FX-03 | Death/combat effects (blood, hit sparks) | Types 0x30-0x3F. Death (0x30), BloodSpray (0x32), HitSpark (0x33), Knockback (0x34). Need: spawn hooks in combat/death code paths. |
| FX-04 | Construction/destruction building effects | Types 0x50-0x57. Construction dust (0x50), Destruction collapse (0x51), BuildingFire (0x52). Need: spawn hooks in building state machine transitions. |
| FX-05 | Effect attachment to moving objects | Effect.target field (entity ID), Effect_AttachToEntity. Need: per-frame position sync from target object, detach on target death. |
</phase_requirements>

## Standard Stack

### Core (already in use)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| wgpu | 28.0 | GPU rendering | Already in Cargo.toml, HUD pipeline exists |
| winit | 0.30 | Window/input | Already in Cargo.toml |
| bytemuck | 1.x | Pod/Zeroable for GPU buffers | Already in Cargo.toml |
| cgmath | 0.18 | Math types | Already in Cargo.toml |

### No new dependencies needed

All Phase 3 features are built on existing infrastructure:
- HUD rendering: extend existing HudRenderer (draw_rect, draw_text, draw_sprite, push_quad)
- Effect rendering: add to world render pass or separate effect pass using existing wgpu pipeline patterns
- Font data: parse binary font files using existing pattern (see psfb.rs for binary parsing)
- String table: simple binary parser, no external crate needed

## Architecture Patterns

### Recommended Project Structure
```
src/
  engine/
    effects/
      mod.rs          # EffectPool, Effect struct, EffectType enum
      types.rs         # 93 effect type definitions + categories
      update.rs        # Effect_UpdateAll loop + per-type handlers
      spawn.rs         # Effect_SpawnAt, Effect_AttachToEntity
    state/
      tribe.rs         # Already has TribeData (mana, population, etc.)
  render/
    hud/
      mod.rs           # Existing HudRenderer -- extend HudState
    effects.rs         # Effect rendering (billboard sprites in world space)
  data/
    font.rs            # Original font file parser (12/16/24pt bitmap)
    strings.rs         # Language string table parser (lang00.dat)
```

### Pattern 1: HudState Data Contract (EXISTING -- extend it)
**What:** Game logic produces a `HudState` struct each frame; renderer consumes it with zero game-logic knowledge.
**When to use:** All HUD additions.
**Example (extend existing):**
```rust
pub struct HudState {
    pub active_tab: HudTab,
    pub minimap: MinimapData,
    pub panel_entries: Vec<PanelEntry>,
    pub tribe_populations: Vec<TribePopulation>,
    pub level_num: u32,
    pub frame_count: u64,
    // NEW fields for Phase 3:
    pub player_mana: u32,
    pub player_max_mana: u32,      // 1_000_000
    pub player_population: u32,
    pub player_max_population: u16,
    pub spell_cooldowns: Vec<SpellCooldown>,
    pub selected_info: Option<SelectedEntityInfo>,
    pub health_bars: Vec<HealthBarEntry>,
    pub camera_viewport: MinimapViewport,  // for viewport rect on minimap
}
```

### Pattern 2: Effect Pool with Free List (matches existing ObjectPool pattern)
**What:** Pre-allocated array of 512 Effect slots, LIFO free list for O(1) alloc/free, linked list for active iteration.
**When to use:** Effect system core.
**Example:**
```rust
pub struct Effect {
    pub effect_type: u8,     // 0x00-0x5C (93 types)
    pub state: u8,           // current state in state machine
    pub flags: u8,           // GRAVITY, LOOP, etc.
    pub owner: u8,           // tribe index 0-3
    pub x: i32,              // world position
    pub y: i32,
    pub z: i32,              // height
    pub frame: i16,          // animation frame
    pub max_frame: i16,
    pub velocity_x: i32,     // fixed-point velocity
    pub velocity_y: i32,
    pub velocity_z: i32,
    pub scale: i16,          // render scale (0x100 = 100%)
    pub alpha: i16,          // transparency
    pub target: Option<u32>, // attached entity ID
    pub damage: i32,
    pub radius: i32,
    pub duration: i32,
    pub color: u32,          // RGBA packed
}

pub const MAX_EFFECTS: usize = 512;

pub struct EffectPool {
    slots: Vec<Effect>,          // pre-allocated 512
    free_list: Vec<u16>,         // LIFO free indices
    active_head: Option<u16>,    // linked list of active effects
    active_count: u32,
}
```

### Pattern 3: World-Space Health Bars
**What:** Health bars positioned above entities by projecting world coordinates to screen space, then rendered in the HUD pass.
**When to use:** HUD-06.
**Key insight from spec:** Original uses depth-bucketed sprite commands (type 0x0F-0x12) sorted by distance. In our wgpu renderer, compute screen positions from world-to-screen projection, then render as HUD quads.
```rust
pub struct HealthBarEntry {
    pub screen_x: f32,
    pub screen_y: f32,
    pub health_fraction: f32,  // 0.0-1.0
    pub bar_type: HealthBarType,  // Unit, Building, Mana, Training
}
```

### Anti-Patterns to Avoid
- **Putting game logic in the renderer:** All mana/population/selection queries go in `build_hud_state()`, not in `draw_hud()`. The renderer just draws what it's told.
- **Recreating GPU resources every frame:** The minimap currently recreates its texture every frame (GpuTexture::new_2d). For health bars and effects, reuse buffers and update via queue.write_buffer.
- **Hand-rolling font rendering from original binary data for v1:** The existing 8x8 bitmap font works. Loading original .fon files is a nice-to-have for visual fidelity but not blocking.
- **Implementing all 93 effect types at once:** Start with the pool infrastructure + a few key types (burn, blast, death, construction). Add types incrementally.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| String table parsing | Custom UTF-8 decoder | Simple binary reader (offset table + null-terminated ASCII) | Format is trivial: count + offset array + null-terminated strings |
| Effect state machine dispatch | Giant match block in one function | Per-category handler functions indexed by type | Keeps code modular, matches original's function pointer table |
| Health bar screen positioning | Custom projection math | Reuse existing Camera::world_to_screen() | Camera already does MVP projection |
| Minimap click-to-move | Custom coordinate transform | Linear mapping: click_pos / minimap_size * 128 = cell coords | Direct proportional mapping, no complex math |

## Common Pitfalls

### Pitfall 1: Minimap Coordinate Wrapping
**What goes wrong:** Minimap click coordinates don't account for toroidal world wrapping when calculating camera target position.
**Why it happens:** The world wraps at 128 cells; camera position must wrap too.
**How to avoid:** Apply the same wrapping math used elsewhere: `pos & 0x7F` for cell coordinates, world coords mod (128 * 128).
**Warning signs:** Camera jumps to wrong position when clicking near minimap edges.

### Pitfall 2: Health Bar Z-Fighting with World Geometry
**What goes wrong:** Health bars rendered in the world pass get occluded by terrain or z-fight.
**Why it happens:** Health bars are screen-space overlays, not 3D geometry.
**How to avoid:** Render health bars in the HUD pass (after world pass), positioned using projected screen coordinates. The existing HUD pass has no depth buffer (depth_stencil: None).
**Warning signs:** Health bars flicker or disappear behind terrain.

### Pitfall 3: Effect Pool Exhaustion
**What goes wrong:** Trying to spawn an effect when all 512 slots are full causes silent failure or panic.
**Why it happens:** Intense combat can generate many effects simultaneously.
**How to avoid:** Effect_SpawnAt returns Option<EffectId>. Callers handle None gracefully. Original binary just returns 0 (failure) when pool is full.
**Warning signs:** Effects stop appearing during heavy combat.

### Pitfall 4: Borrow Checker Conflicts in Effect Updates
**What goes wrong:** Effect update needs to read entity positions (for attached effects) while also mutating the effect pool.
**Why it happens:** Same pattern seen in Phase 2 with DeferredAction and ManaTickBridge.
**How to avoid:** Use the same two-phase collect-then-process pattern: collect position updates from entities first, then apply to effects.
**Warning signs:** Cannot compile due to simultaneous mutable/immutable borrows.

### Pitfall 5: HudState Growing Too Large
**What goes wrong:** build_hud_state() allocates Vecs every frame, causing GC pressure and slowdown.
**Why it happens:** Health bars for all visible units = potentially hundreds of entries per frame.
**How to avoid:** Pre-allocate Vec capacity, or use a persistent HudState that's cleared + refilled each frame instead of reconstructed.
**Warning signs:** Frame time spikes during large battles.

## Code Examples

### Extend HudState for Mana/Population (HUD-03, HUD-04)
```rust
// In build_hud_state():
let player_tribe = &self.tribes.tribes[0]; // tribe 0 = player
HudState {
    // ... existing fields ...
    player_mana: player_tribe.mana,
    player_max_mana: 1_000_000,
    player_population: player_tribe.population,
    player_max_population: player_tribe.max_population,
}

// In draw_hud() -- mana bar:
let mana_frac = hud_state.player_mana as f32 / hud_state.player_max_mana as f32;
let bar_w = layout.sidebar_w - layout.mm_pad * 2.0;
let bar_h = 8.0 * layout.scale_y;
let bar_y = layout.screen_h - bar_h - 4.0;
hud.draw_rect(layout.mm_pad, bar_y, bar_w, bar_h, [0.1, 0.1, 0.2, 0.8]); // bg
hud.draw_rect(layout.mm_pad, bar_y, bar_w * mana_frac, bar_h, [0.3, 0.5, 1.0, 0.9]); // fill
```

### Minimap Camera Viewport Rectangle (HUD-01)
```rust
pub struct MinimapViewport {
    pub cam_cell_x: f32,  // camera center in cell coords
    pub cam_cell_y: f32,
    pub view_width_cells: f32,  // visible area width in cells
    pub view_height_cells: f32,
}

// In draw_hud(), after minimap texture:
let vp = &hud_state.camera_viewport;
let cell_to_px = layout.mm_size / 128.0;
let rx = layout.mm_x + vp.cam_cell_x * cell_to_px - vp.view_width_cells * cell_to_px / 2.0;
let ry = layout.mm_y + vp.cam_cell_y * cell_to_px - vp.view_height_cells * cell_to_px / 2.0;
let rw = vp.view_width_cells * cell_to_px;
let rh = vp.view_height_cells * cell_to_px;
// Draw viewport rectangle outline (4 thin rects)
let border = 1.0;
hud.draw_rect(rx, ry, rw, border, [1.0, 1.0, 1.0, 0.8]);           // top
hud.draw_rect(rx, ry + rh - border, rw, border, [1.0, 1.0, 1.0, 0.8]); // bottom
hud.draw_rect(rx, ry, border, rh, [1.0, 1.0, 1.0, 0.8]);           // left
hud.draw_rect(rx + rw - border, ry, border, rh, [1.0, 1.0, 1.0, 0.8]); // right
```

### Effect Pool Core (FX-01)
```rust
impl EffectPool {
    pub fn new() -> Self {
        let mut slots = Vec::with_capacity(MAX_EFFECTS);
        let mut free_list = Vec::with_capacity(MAX_EFFECTS);
        for i in 0..MAX_EFFECTS {
            slots.push(Effect::default());
            free_list.push(i as u16);
        }
        Self { slots, free_list, active_head: None, active_count: 0 }
    }

    pub fn spawn(&mut self, effect_type: u8, x: i32, y: i32, z: i32, owner: u8) -> Option<u16> {
        let idx = self.free_list.pop()?;
        let effect = &mut self.slots[idx as usize];
        *effect = Effect::default();
        effect.effect_type = effect_type;
        effect.x = x;
        effect.y = y;
        effect.z = z;
        effect.owner = owner;
        effect.state = 0; // Init state
        // Init frame count from type table
        self.active_count += 1;
        Some(idx)
    }

    pub fn destroy(&mut self, idx: u16) {
        self.free_list.push(idx);
        self.active_count -= 1;
    }

    pub fn update_all(&mut self) {
        // Iterate active effects, update position, advance frame, call type handler
        for i in 0..MAX_EFFECTS {
            let effect = &mut self.slots[i];
            if effect.state == 0xFF { continue; } // inactive sentinel
            // Position update
            effect.x += effect.velocity_x >> 8;
            effect.y += effect.velocity_y >> 8;
            effect.z += effect.velocity_z >> 8;
            // Gravity
            if effect.flags & EFFECT_GRAVITY != 0 {
                effect.velocity_z -= GRAVITY_ACCEL;
            }
            // Frame advance
            effect.frame += 1;
            if effect.frame >= effect.max_frame {
                if effect.flags & EFFECT_LOOP != 0 {
                    effect.frame = 0;
                } else {
                    effect.state = 0xFF; // mark for cleanup
                }
            }
        }
    }
}
```

### String Table Loading (HUD-08)
```rust
/// Load language string table from LANGUAGE/lang00.dat
pub fn load_string_table(data: &[u8]) -> Vec<String> {
    if data.len() < 4 { return Vec::new(); }
    let count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let mut strings = Vec::with_capacity(count);
    // Offsets start at byte 4, each is 4 bytes
    let offsets_end = 4 + count * 4;
    if data.len() < offsets_end { return Vec::new(); }
    for i in 0..count {
        let off_pos = 4 + i * 4;
        let offset = u32::from_le_bytes([
            data[off_pos], data[off_pos+1], data[off_pos+2], data[off_pos+3]
        ]) as usize;
        // Read null-terminated string
        let mut end = offset;
        while end < data.len() && data[end] != 0 { end += 1; }
        let s = String::from_utf8_lossy(&data[offset..end]).to_string();
        strings.push(s);
    }
    strings
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| 8x8 bitmap font only | Need 12/16/24pt from original .fon data | Phase 3 | Text readability at different sizes |
| Static panel entries (hardcoded strings) | Dynamic entries from game state | Phase 3 | Live spell cooldowns, selection info |
| No effects system | 512-slot pool with state machine | Phase 3 | Visual feedback for all game events |
| Minimap display-only | Interactive click-to-move | Phase 3 | Core game navigation feature |

## Open Questions

1. **Original font file format details**
   - What we know: 12pt = 2 bytes/row x 16 rows = 32 bytes/char; 24pt = 3 bytes/row x 24 rows = 72 bytes/char. Files are font12j.fon, font16j.fon, font24j.fon.
   - What's unclear: Exact header format, character index layout, whether we need the original fonts or can use the 8x8 bitmap scaled up.
   - Recommendation: Start with the 8x8 bitmap (already working) at scaled sizes. Load original fonts as a follow-up if text quality is insufficient. For v1 English-only, the 8x8 bitmap at 12/16/24pt scale is likely acceptable.

2. **Effect sprite assets**
   - What we know: Effects use sprite banks (0x3a-0x3d for blast, 0x29 for swarm, 0x3c for explosion, etc.).
   - What's unclear: Which sprite files contain effect sprites and whether they're already being loaded.
   - Recommendation: Check if plspanel.spr or other loaded PSFB containers have effect sprites. If not, identify the correct SPR files to load. Can start with colored rectangles/circles as placeholder effects.

3. **Effect rendering approach**
   - What we know: Original renders effects as depth-bucketed sprites in the world pass.
   - What's unclear: Whether to render as billboarded 3D sprites in the world pass or as screen-projected 2D sprites in the HUD pass.
   - Recommendation: Render as screen-projected 2D sprites in HUD pass (simpler, matches health bar approach). World-space depth sorting can be added later if visual quality demands it.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (#[cfg(test)] + cargo test) |
| Config file | Cargo.toml |
| Quick run command | `cargo test --lib` |
| Full suite command | `cargo test` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| HUD-01 | Minimap viewport rect + click-to-cell mapping | unit | `cargo test hud::tests::minimap_click -x` | Partial (mod.rs has tests, need new) |
| HUD-02 | Spell cooldown data generation | unit | `cargo test hud::tests::spell_cooldown -x` | No - Wave 0 |
| HUD-03 | Mana bar fraction calculation | unit | `cargo test hud::tests::mana_bar -x` | No - Wave 0 |
| HUD-04 | Population display values | unit | `cargo test hud::tests::population -x` | No - Wave 0 |
| HUD-05 | Selection info extraction | unit | `cargo test hud::tests::selection_info -x` | No - Wave 0 |
| HUD-06 | Health bar screen projection | unit | `cargo test hud::tests::health_bar -x` | No - Wave 0 |
| HUD-07 | Font glyph lookup at different sizes | unit | `cargo test hud::tests::font_sizes -x` | Partial (font rgba tests exist) |
| HUD-08 | String table parse + lookup | unit | `cargo test data::strings::tests -x` | No - Wave 0 |
| FX-01 | Effect pool alloc/free/capacity | unit | `cargo test engine::effects::tests::pool -x` | No - Wave 0 |
| FX-02 | Spell effect init + state transitions | unit | `cargo test engine::effects::tests::spell -x` | No - Wave 0 |
| FX-03 | Death/combat effect spawn | unit | `cargo test engine::effects::tests::combat -x` | No - Wave 0 |
| FX-04 | Building effect spawn on state change | unit | `cargo test engine::effects::tests::building -x` | No - Wave 0 |
| FX-05 | Effect attachment position sync | unit | `cargo test engine::effects::tests::attach -x` | No - Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --lib`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/engine/effects/mod.rs` -- Effect struct, EffectPool, EffectType enum, basic alloc/free tests
- [ ] `src/data/strings.rs` -- string table parser with tests
- [ ] `src/data/font.rs` -- font data parser (optional if using scaled 8x8)
- [ ] Extend existing `src/render/hud/mod.rs` tests for new HudState fields

## Sources

### Primary (HIGH confidence)
- `docs/specs/ui_and_input.md` -- HUD rendering order, minimap system, spell panel, health bars, selection rings, font system (924 lines of reverse-engineered documentation)
- `docs/specs/water_and_effects.md` -- 93 effect types, Effect struct (64 bytes), 512-slot pool, state machine, particle distribution (890 lines)
- `src/render/hud/mod.rs` -- Existing HudRenderer implementation (1306 lines)
- `src/render/app.rs` -- build_hud_state() and draw_hud() at lines 414-1802
- `src/engine/state/tribe.rs` -- TribeData with mana, population fields
- `src/engine/units/selection.rs` -- SelectionState with Vec<UnitId>
- `src/engine/units/unit.rs` -- Unit struct with health, max_health, subtype

### Secondary (MEDIUM confidence)
- `things-to-implement.md` sections 10 and 13 -- Implementation status tracking

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all libraries already in use, no new deps
- Architecture: HIGH -- extending well-established patterns (HudState data contract, ObjectPool-style free list)
- Pitfalls: HIGH -- based on actual codebase patterns (DeferredAction, ManaTickBridge borrow-checker solutions)
- Effect system: MEDIUM -- 93 effect types documented but only need a subset for v1; exact sprite asset mapping uncertain

**Research date:** 2026-03-18
**Valid until:** 2026-04-17 (stable domain, project-specific)
