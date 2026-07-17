// HUD data types, layout computation, rendering helpers, and GPU renderer.
pub mod layout;

use std::collections::HashMap;
use std::sync::Arc;

use crate::data::psfb::ContainerPSFB;
use crate::render::gpu::buffer::GpuBuffer;
use crate::render::gpu::texture::GpuTexture;
// FontData provides the multi-size glyph API (8x8, 16x16, 24x24 via integer scaling).
// Currently draw_text_sized() delegates to the atlas-based draw_text() which already
// supports arbitrary pixel sizes. FontData will be used directly when loading the
// original .fon files (font12j/font16j/font24j) for higher-quality scaled text.
#[allow(unused_imports)]
use crate::data::font::FontData;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct HudVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

#[derive(Clone, Debug)]
pub struct SpriteRegion {
    pub u0: f32,
    pub v0: f32,
    pub u1: f32,
    pub v1: f32,
    pub width: u16,
    pub height: u16,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HudTab {
    Buildings,
    Spells,
    Units,
}

/// Pre-computed layout dimensions for the HUD, derived from screen size.
/// Single source of truth — used by both rendering and input handling.
#[derive(Clone, Debug)]
pub struct HudLayout {
    pub screen_w: f32,
    pub screen_h: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub sidebar_w: f32,
    pub font_scale: f32,
    pub small_font: f32,
    pub mm_pad: f32,
    pub mm_x: f32,
    pub mm_y: f32,
    pub mm_size: f32,
    pub mm_w: f32,
    pub mm_h: f32,
    pub mana_bar_y: f32,
    pub mana_bar_h: f32,
    pub pop_y: f32,
    pub pop_h: f32,
    pub tab_y: f32,
    pub tab_h: f32,
    pub tab_w: f32,
    /// Tab draw positions in visual order: construction, spells, followers.
    pub tab_xs: [f32; 3],
    pub status_y: f32,
    pub status_h: f32,
    pub panel_y: f32,
    pub construction_cell_w: f32,
    pub construction_cell_h: f32,
    pub line_h: f32,
}

// ---------------------------------------------------------------------------
// Data contract: game logic → HUD
// ---------------------------------------------------------------------------

/// Data the game logic provides to the HUD each frame.
/// The HUD renders whatever is in here — no game logic knowledge.
pub struct HudState {
    pub active_tab: HudTab,
    pub minimap: MinimapData,
    pub panel_entries: Vec<PanelEntry>,
    pub tribe_populations: Vec<TribePopulation>,
    pub level_num: u32,
    pub frame_count: u64,
    // Phase 3: player resource and spell cooldown data
    pub player_mana: u32,
    pub player_max_mana: u32,
    pub player_population: u32,
    pub player_max_population: u16,
    pub spell_cooldowns: Vec<SpellCooldown>,
    pub spell_charges: [u8; 16],
    pub selected_info: Option<SelectedEntityInfo>,
    pub health_bars: Vec<HealthBarEntry>,
}

pub struct MinimapData {
    pub heights: [[u16; 128]; 128],
    /// Terrain generated from the original level's BIGF0 and palette.  This
    /// is populated when native resources are available; `heights` remains a
    /// portable fallback for test and no-asset runs.
    pub native_terrain_rgba: Option<Arc<[u8]>>,
    /// The level palette that the original object pass uses for its marker
    /// indices.  It stays separate from the expanded terrain so people and
    /// buildings retain their distinct native palette colours.
    pub native_palette: Option<Arc<[u8]>>,
    pub dots: Vec<MinimapDot>,
}

/// Object classes rendered by the original minimap object pass.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MinimapMarkerKind {
    Person,
    Building,
    WildPerson,
}

pub struct MinimapDot {
    pub cell_x: u8,
    pub cell_y: u8,
    pub tribe_index: u8,
    pub kind: MinimapMarkerKind,
}

pub struct PanelEntry {
    pub label: String,
    pub color: [f32; 4],
}

pub struct TribePopulation {
    pub tribe_index: u8,
    pub count: u32,
    pub color: [f32; 4],
}

/// Selected entity info for sidebar detail panel.
pub struct SelectedEntityInfo {
    pub name: String,
    pub health: u16,
    pub max_health: u16,
    pub subtype: u8,
    pub tribe_index: u8,
    pub extra_lines: Vec<String>,
}

/// Health bar entry for world-projected health bars in the HUD overlay.
pub struct HealthBarEntry {
    pub screen_x: f32,        // screen-space center X
    pub screen_y: f32,        // screen-space top Y (above entity)
    pub health_fraction: f32, // 0.0-1.0
    pub bar_type: HealthBarType,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HealthBarType {
    Unit,
    Building,
}

/// Spell cooldown state for HUD rendering.
/// Phase 4 will populate from SpellSystem cooldown timers.
pub struct SpellCooldown {
    pub spell_index: u8,         // 0-15 matching spell panel order
    pub cooldown_remaining: u32, // ticks remaining (0 = ready)
    pub cooldown_total: u32,     // total cooldown duration
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const FONT_GLYPH_W: u32 = 8;
pub const FONT_GLYPH_H: u32 = 8;
pub const FONT_COLS: u32 = 16;
pub const FONT_ROWS: u32 = 6;
pub const FONT_ATLAS_W: u32 = FONT_COLS * FONT_GLYPH_W; // 128
pub const FONT_ATLAS_H: u32 = FONT_ROWS * FONT_GLYPH_H; // 48

/// Palette indices loaded by the original `Minimap_RenderObjects` pass.
/// The executable stores these as 5-byte tribe records at `0x5A17A9`:
/// person at byte zero, building at byte one.
pub const MINIMAP_PERSON_PALETTE_INDICES: [u8; 4] = [0xDF, 0xF6, 0xEF, 0xE5];
pub const MINIMAP_BUILDING_PALETTE_INDICES: [u8; 4] = [0xDA, 0xF2, 0xEC, 0xE2];
pub const MINIMAP_WILD_PERSON_PALETTE_INDEX: u8 = 0xBF;

// The source palette is supplied at runtime.  These values are the matching
// entries from the shipped level palettes and only cover no-asset/test runs.
const MINIMAP_PERSON_FALLBACK_COLORS: [[u8; 3]; 4] = [
    [0x87, 0x8B, 0xEB],
    [0xC7, 0x73, 0x4B],
    [0xFB, 0xD7, 0x5F],
    [0x3F, 0xCF, 0x77],
];
const MINIMAP_BUILDING_FALLBACK_COLORS: [[u8; 3]; 4] = [
    [0x23, 0x33, 0x6F],
    [0x5F, 0x07, 0x07],
    [0xA3, 0x77, 0x13],
    [0x17, 0x67, 0x3F],
];
const MINIMAP_WILD_PERSON_FALLBACK_COLOR: [u8; 3] = [0x7F, 0x77, 0x57];

/// Native minimap water sampled from the uniform ocean in the owner's
/// original-HUD capture: RGB `#00556B`. The minimap is dynamic, but its base
/// water color is not an arbitrary remake tint.
pub const MINIMAP_WATER_COLOR: [u8; 3] = [0x00, 0x55, 0x6B];

/// Tribe colors for HUD text overlay (RGBA, 0.0-1.0).
pub const HUD_TRIBE_COLORS: [[f32; 4]; 4] = [
    [0.3, 0.5, 1.0, 0.9], // Blue
    [1.0, 0.3, 0.3, 0.9], // Red
    [1.0, 1.0, 0.3, 0.9], // Yellow
    [0.3, 1.0, 0.3, 0.9], // Green
];

// ---------------------------------------------------------------------------
// HUD data helpers
// ---------------------------------------------------------------------------

/// Compute mana bar fill fraction, clamped to [0.0, 1.0].
pub fn compute_mana_fraction(mana: u32, max_mana: u32) -> f32 {
    if max_mana == 0 {
        return 0.0;
    }
    (mana as f32 / max_mana as f32).min(1.0)
}

/// Convert a minimap pixel click to cell coordinates (0-127).
pub fn minimap_click_to_cell(
    click_x: f32,
    click_y: f32,
    mm_x: f32,
    mm_y: f32,
    mm_w: f32,
    mm_h: f32,
) -> (f32, f32) {
    let cell_x = ((click_x - mm_x) / mm_w * 128.0).clamp(0.0, 127.0);
    let cell_y = ((click_y - mm_y) / mm_h * 128.0).clamp(0.0, 127.0);
    (cell_x, cell_y)
}

/// Compute shortest toroidal delta on a 128-cell wrapping map.
pub fn toroidal_delta(from: f32, to: f32) -> f32 {
    let raw = to - from;
    if raw > 64.0 {
        raw - 128.0
    } else if raw < -64.0 {
        raw + 128.0
    } else {
        raw
    }
}

/// Map unit subtype id to display name.
pub fn unit_subtype_name(subtype: u8) -> &'static str {
    match subtype {
        1 => "Wild",
        2 => "Brave",
        3 => "Warrior",
        4 => "Preacher",
        5 => "Spy",
        6 => "Super Warrior",
        7 => "Shaman",
        _ => "Unknown",
    }
}

/// 8x8 bitmap font for ASCII 32..127 (96 glyphs).
/// Each glyph is 8 bytes (one byte per row, MSB = leftmost pixel).
pub const FONT_8X8: [[u8; 8]; 96] = {
    let mut f = [[0u8; 8]; 96];
    f[0] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; // Space (32)
    f[1] = [0x18, 0x18, 0x18, 0x18, 0x18, 0x00, 0x18, 0x00]; // !
    f[2] = [0x6C, 0x6C, 0x6C, 0x00, 0x00, 0x00, 0x00, 0x00]; // "
    f[3] = [0x6C, 0x6C, 0xFE, 0x6C, 0xFE, 0x6C, 0x6C, 0x00]; // #
    f[4] = [0x18, 0x7E, 0xC0, 0x7C, 0x06, 0xFC, 0x18, 0x00]; // $
    f[5] = [0x00, 0xC6, 0xCC, 0x18, 0x30, 0x66, 0xC6, 0x00]; // %
    f[6] = [0x38, 0x6C, 0x38, 0x76, 0xDC, 0xCC, 0x76, 0x00]; // &
    f[7] = [0x18, 0x18, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00]; // '
    f[8] = [0x0C, 0x18, 0x30, 0x30, 0x30, 0x18, 0x0C, 0x00]; // (
    f[9] = [0x30, 0x18, 0x0C, 0x0C, 0x0C, 0x18, 0x30, 0x00]; // )
    f[10] = [0x00, 0x66, 0x3C, 0xFF, 0x3C, 0x66, 0x00, 0x00]; // *
    f[11] = [0x00, 0x18, 0x18, 0x7E, 0x18, 0x18, 0x00, 0x00]; // +
    f[12] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x30]; // ,
    f[13] = [0x00, 0x00, 0x00, 0x7E, 0x00, 0x00, 0x00, 0x00]; // -
    f[14] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00]; // .
    f[15] = [0x06, 0x0C, 0x18, 0x30, 0x60, 0xC0, 0x80, 0x00]; // /
    f[16] = [0x7C, 0xC6, 0xCE, 0xD6, 0xE6, 0xC6, 0x7C, 0x00]; // 0
    f[17] = [0x18, 0x38, 0x18, 0x18, 0x18, 0x18, 0x7E, 0x00]; // 1
    f[18] = [0x7C, 0xC6, 0x06, 0x1C, 0x30, 0x66, 0xFE, 0x00]; // 2
    f[19] = [0x7C, 0xC6, 0x06, 0x3C, 0x06, 0xC6, 0x7C, 0x00]; // 3
    f[20] = [0x1C, 0x3C, 0x6C, 0xCC, 0xFE, 0x0C, 0x1E, 0x00]; // 4
    f[21] = [0xFE, 0xC0, 0xFC, 0x06, 0x06, 0xC6, 0x7C, 0x00]; // 5
    f[22] = [0x38, 0x60, 0xC0, 0xFC, 0xC6, 0xC6, 0x7C, 0x00]; // 6
    f[23] = [0xFE, 0xC6, 0x0C, 0x18, 0x30, 0x30, 0x30, 0x00]; // 7
    f[24] = [0x7C, 0xC6, 0xC6, 0x7C, 0xC6, 0xC6, 0x7C, 0x00]; // 8
    f[25] = [0x7C, 0xC6, 0xC6, 0x7E, 0x06, 0x0C, 0x78, 0x00]; // 9
    f[26] = [0x00, 0x18, 0x18, 0x00, 0x00, 0x18, 0x18, 0x00]; // :
    f[27] = [0x00, 0x18, 0x18, 0x00, 0x00, 0x18, 0x18, 0x30]; // ;
    f[28] = [0x0C, 0x18, 0x30, 0x60, 0x30, 0x18, 0x0C, 0x00]; // <
    f[29] = [0x00, 0x00, 0x7E, 0x00, 0x00, 0x7E, 0x00, 0x00]; // =
    f[30] = [0x60, 0x30, 0x18, 0x0C, 0x18, 0x30, 0x60, 0x00]; // >
    f[31] = [0x7C, 0xC6, 0x0C, 0x18, 0x18, 0x00, 0x18, 0x00]; // ?
    f[32] = [0x7C, 0xC6, 0xDE, 0xDE, 0xDE, 0xC0, 0x78, 0x00]; // @
    f[33] = [0x38, 0x6C, 0xC6, 0xC6, 0xFE, 0xC6, 0xC6, 0x00]; // A
    f[34] = [0xFC, 0x66, 0x66, 0x7C, 0x66, 0x66, 0xFC, 0x00]; // B
    f[35] = [0x3C, 0x66, 0xC0, 0xC0, 0xC0, 0x66, 0x3C, 0x00]; // C
    f[36] = [0xF8, 0x6C, 0x66, 0x66, 0x66, 0x6C, 0xF8, 0x00]; // D
    f[37] = [0xFE, 0x62, 0x68, 0x78, 0x68, 0x62, 0xFE, 0x00]; // E
    f[38] = [0xFE, 0x62, 0x68, 0x78, 0x68, 0x60, 0xF0, 0x00]; // F
    f[39] = [0x3C, 0x66, 0xC0, 0xC0, 0xCE, 0x66, 0x3E, 0x00]; // G
    f[40] = [0xC6, 0xC6, 0xC6, 0xFE, 0xC6, 0xC6, 0xC6, 0x00]; // H
    f[41] = [0x3C, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00]; // I
    f[42] = [0x1E, 0x0C, 0x0C, 0x0C, 0xCC, 0xCC, 0x78, 0x00]; // J
    f[43] = [0xE6, 0x66, 0x6C, 0x78, 0x6C, 0x66, 0xE6, 0x00]; // K
    f[44] = [0xF0, 0x60, 0x60, 0x60, 0x62, 0x66, 0xFE, 0x00]; // L
    f[45] = [0xC6, 0xEE, 0xFE, 0xFE, 0xD6, 0xC6, 0xC6, 0x00]; // M
    f[46] = [0xC6, 0xE6, 0xF6, 0xDE, 0xCE, 0xC6, 0xC6, 0x00]; // N
    f[47] = [0x7C, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x7C, 0x00]; // O
    f[48] = [0xFC, 0x66, 0x66, 0x7C, 0x60, 0x60, 0xF0, 0x00]; // P
    f[49] = [0x7C, 0xC6, 0xC6, 0xC6, 0xD6, 0xDE, 0x7C, 0x06]; // Q
    f[50] = [0xFC, 0x66, 0x66, 0x7C, 0x6C, 0x66, 0xE6, 0x00]; // R
    f[51] = [0x7C, 0xC6, 0xE0, 0x7C, 0x0E, 0xC6, 0x7C, 0x00]; // S
    f[52] = [0x7E, 0x7E, 0x5A, 0x18, 0x18, 0x18, 0x3C, 0x00]; // T
    f[53] = [0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x7C, 0x00]; // U
    f[54] = [0xC6, 0xC6, 0xC6, 0xC6, 0x6C, 0x38, 0x10, 0x00]; // V
    f[55] = [0xC6, 0xC6, 0xD6, 0xFE, 0xFE, 0xEE, 0xC6, 0x00]; // W
    f[56] = [0xC6, 0x6C, 0x38, 0x38, 0x38, 0x6C, 0xC6, 0x00]; // X
    f[57] = [0x66, 0x66, 0x66, 0x3C, 0x18, 0x18, 0x3C, 0x00]; // Y
    f[58] = [0xFE, 0xC6, 0x8C, 0x18, 0x32, 0x66, 0xFE, 0x00]; // Z
    f[59] = [0x3C, 0x30, 0x30, 0x30, 0x30, 0x30, 0x3C, 0x00]; // [
    f[60] = [0xC0, 0x60, 0x30, 0x18, 0x0C, 0x06, 0x02, 0x00]; // backslash
    f[61] = [0x3C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x3C, 0x00]; // ]
    f[62] = [0x10, 0x38, 0x6C, 0xC6, 0x00, 0x00, 0x00, 0x00]; // ^
    f[63] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF]; // _
    f[64] = [0x30, 0x18, 0x0C, 0x00, 0x00, 0x00, 0x00, 0x00]; // `
    f[65] = [0x00, 0x00, 0x78, 0x0C, 0x7C, 0xCC, 0x76, 0x00]; // a
    f[66] = [0xE0, 0x60, 0x7C, 0x66, 0x66, 0x66, 0xDC, 0x00]; // b
    f[67] = [0x00, 0x00, 0x7C, 0xC6, 0xC0, 0xC6, 0x7C, 0x00]; // c
    f[68] = [0x1C, 0x0C, 0x7C, 0xCC, 0xCC, 0xCC, 0x76, 0x00]; // d
    f[69] = [0x00, 0x00, 0x7C, 0xC6, 0xFE, 0xC0, 0x7C, 0x00]; // e
    f[70] = [0x1C, 0x36, 0x30, 0x78, 0x30, 0x30, 0x78, 0x00]; // f
    f[71] = [0x00, 0x00, 0x76, 0xCC, 0xCC, 0x7C, 0x0C, 0xF8]; // g
    f[72] = [0xE0, 0x60, 0x6C, 0x76, 0x66, 0x66, 0xE6, 0x00]; // h
    f[73] = [0x18, 0x00, 0x38, 0x18, 0x18, 0x18, 0x3C, 0x00]; // i
    f[74] = [0x06, 0x00, 0x06, 0x06, 0x06, 0x66, 0x66, 0x3C]; // j
    f[75] = [0xE0, 0x60, 0x66, 0x6C, 0x78, 0x6C, 0xE6, 0x00]; // k
    f[76] = [0x38, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00]; // l
    f[77] = [0x00, 0x00, 0xEC, 0xFE, 0xD6, 0xD6, 0xD6, 0x00]; // m
    f[78] = [0x00, 0x00, 0xDC, 0x66, 0x66, 0x66, 0x66, 0x00]; // n
    f[79] = [0x00, 0x00, 0x7C, 0xC6, 0xC6, 0xC6, 0x7C, 0x00]; // o
    f[80] = [0x00, 0x00, 0xDC, 0x66, 0x66, 0x7C, 0x60, 0xF0]; // p
    f[81] = [0x00, 0x00, 0x76, 0xCC, 0xCC, 0x7C, 0x0C, 0x1E]; // q
    f[82] = [0x00, 0x00, 0xDC, 0x76, 0x60, 0x60, 0xF0, 0x00]; // r
    f[83] = [0x00, 0x00, 0x7E, 0xC0, 0x7C, 0x06, 0xFC, 0x00]; // s
    f[84] = [0x30, 0x30, 0x7C, 0x30, 0x30, 0x36, 0x1C, 0x00]; // t
    f[85] = [0x00, 0x00, 0xCC, 0xCC, 0xCC, 0xCC, 0x76, 0x00]; // u
    f[86] = [0x00, 0x00, 0xC6, 0xC6, 0xC6, 0x6C, 0x38, 0x00]; // v
    f[87] = [0x00, 0x00, 0xC6, 0xD6, 0xD6, 0xFE, 0x6C, 0x00]; // w
    f[88] = [0x00, 0x00, 0xC6, 0x6C, 0x38, 0x6C, 0xC6, 0x00]; // x
    f[89] = [0x00, 0x00, 0xC6, 0xC6, 0xCE, 0x76, 0x06, 0xFC]; // y
    f[90] = [0x00, 0x00, 0xFC, 0x98, 0x30, 0x64, 0xFC, 0x00]; // z
    f[91] = [0x0E, 0x18, 0x18, 0x70, 0x18, 0x18, 0x0E, 0x00]; // {
    f[92] = [0x18, 0x18, 0x18, 0x00, 0x18, 0x18, 0x18, 0x00]; // |
    f[93] = [0x70, 0x18, 0x18, 0x0E, 0x18, 0x18, 0x70, 0x00]; // }
    f[94] = [0x76, 0xDC, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; // ~
    f[95] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; // DEL placeholder
    f
};

// ---------------------------------------------------------------------------
// Pure functions
// ---------------------------------------------------------------------------

/// Generate RGBA pixel data for the 128x48 bitmap font atlas.
pub fn build_font_rgba() -> Vec<u8> {
    let mut rgba = vec![0u8; (FONT_ATLAS_W * FONT_ATLAS_H * 4) as usize];
    for (idx, glyph) in FONT_8X8.iter().enumerate() {
        let col = (idx as u32) % FONT_COLS;
        let row = (idx as u32) / FONT_COLS;
        let ox = col * FONT_GLYPH_W;
        let oy = row * FONT_GLYPH_H;
        for y in 0..8u32 {
            let bits = glyph[y as usize];
            for x in 0..8u32 {
                if bits & (0x80 >> x) != 0 {
                    let px = ox + x;
                    let py = oy + y;
                    let off = ((py * FONT_ATLAS_W + px) * 4) as usize;
                    rgba[off] = 255;
                    rgba[off + 1] = 255;
                    rgba[off + 2] = 255;
                    rgba[off + 3] = 255;
                }
            }
        }
    }
    rgba
}

/// Generate 6 vertices (2 triangles) for a textured quad.
/// Winding: TL→TR→BL, BL→TR→BR.
pub fn generate_quad_vertices(
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    u0: f32,
    v0: f32,
    u1: f32,
    v1: f32,
    color: [f32; 4],
) -> [HudVertex; 6] {
    [
        HudVertex {
            position: [x0, y0],
            uv: [u0, v0],
            color,
        },
        HudVertex {
            position: [x1, y0],
            uv: [u1, v0],
            color,
        },
        HudVertex {
            position: [x0, y1],
            uv: [u0, v1],
            color,
        },
        HudVertex {
            position: [x0, y1],
            uv: [u0, v1],
            color,
        },
        HudVertex {
            position: [x1, y0],
            uv: [u1, v0],
            color,
        },
        HudVertex {
            position: [x1, y1],
            uv: [u1, v1],
            color,
        },
    ]
}

/// Shelf-pack a list of (width, height) items into an atlas of given width.
/// Returns (placements, atlas_height) where placements[i] = (x, y) for item i,
/// and atlas_height is the next power-of-two height that fits everything.
pub fn shelf_pack(items: &[(u16, u16)], atlas_w: u32) -> (Vec<(u32, u32)>, u32) {
    let mut shelf_y: u32 = 0;
    let mut shelf_h: u32 = 0;
    let mut cursor_x: u32 = 0;
    let mut placements = Vec::with_capacity(items.len());

    for &(w, h) in items {
        let sw = w as u32;
        let sh = h as u32;
        if cursor_x + sw > atlas_w {
            shelf_y += shelf_h;
            cursor_x = 0;
            shelf_h = 0;
        }
        placements.push((cursor_x, shelf_y));
        cursor_x += sw + 1; // 1px padding
        shelf_h = shelf_h.max(sh);
    }

    let atlas_h = (shelf_y + shelf_h).next_power_of_two().max(64);
    (placements, atlas_h)
}

/// Convert palette-indexed pixel data to RGBA.
///
/// The original UI banks use either 768-byte RGB palettes or 1024-byte RGBX
/// palettes. `transparent_idx` pixels get alpha=0.
pub fn convert_indexed_to_rgba(indexed: &[u8], palette: &[u8], transparent_idx: u8) -> Vec<u8> {
    let mut rgba = vec![0u8; indexed.len() * 4];
    for (j, &idx) in indexed.iter().enumerate() {
        if idx == transparent_idx {
            rgba[j * 4 + 3] = 0;
        } else {
            let stride = if palette.len() == 768 { 3 } else { 4 };
            let p = (idx as usize) * stride;
            if p + 2 < palette.len() {
                rgba[j * 4] = palette[p];
                rgba[j * 4 + 1] = palette[p + 1];
                rgba[j * 4 + 2] = palette[p + 2];
                rgba[j * 4 + 3] = 255;
            }
        }
    }
    rgba
}

/// Generate 128x128 RGBA minimap texture from terrain heights and unit positions.
pub fn generate_minimap_rgba(data: &MinimapData) -> Vec<u8> {
    let mut rgba = data
        .native_terrain_rgba
        .as_deref()
        .filter(|terrain| terrain.len() == 128 * 128 * 4)
        .map(<[u8]>::to_vec)
        .unwrap_or_else(|| {
            let mut fallback = vec![0u8; 128 * 128 * 4];
            for y in 0..128usize {
                for x in 0..128usize {
                    let off = (y * 128 + x) * 4;
                    let dx = x as f32 + 0.5 - 64.0;
                    let dy = y as f32 + 0.5 - 64.0;
                    if dx * dx + dy * dy > 63.5 * 63.5 {
                        continue;
                    }
                    let h = data.heights[y][x];
                    if h == 0 {
                        fallback[off] = MINIMAP_WATER_COLOR[0];
                        fallback[off + 1] = MINIMAP_WATER_COLOR[1];
                        fallback[off + 2] = MINIMAP_WATER_COLOR[2];
                        fallback[off + 3] = 255;
                    } else {
                        let v = ((h as f32 / 1024.0) * 180.0).min(255.0) as u8;
                        fallback[off] = v / 4;
                        fallback[off + 1] = 40 + v / 2;
                        fallback[off + 2] = v / 6;
                        fallback[off + 3] = 255;
                    }
                }
            }
            fallback
        });
    // The original object pass draws normal people and buildings as 2x2
    // palette-indexed blocks.  Wild people use its separate one-pixel path.
    for dot in &data.dots {
        let cx = (dot.cell_x as usize).min(127);
        let cy = (dot.cell_y as usize).min(127);
        let color = minimap_marker_color(dot, data.native_palette.as_deref());
        let pixels: &[(usize, usize)] = match dot.kind {
            MinimapMarkerKind::WildPerson => &[(cx, cy)],
            MinimapMarkerKind::Person | MinimapMarkerKind::Building => {
                &[(cx, cy), (cx + 1, cy), (cx, cy + 1), (cx + 1, cy + 1)]
            }
        };
        for &(x, y) in pixels {
            if x >= 128 || y >= 128 {
                continue;
            }
            let off = (y * 128 + x) * 4;
            rgba[off] = color[0];
            rgba[off + 1] = color[1];
            rgba[off + 2] = color[2];
            rgba[off + 3] = 255;
        }
    }

    // The original game presents the map through a circular aperture. Keep the
    // texture square for the GPU, but make the corners transparent so the
    // ochre panel frame remains visible around it.
    let radius_sq = 63.0f32 * 63.0;
    for y in 0..128usize {
        for x in 0..128usize {
            let dx = x as f32 + 0.5 - 64.0;
            let dy = y as f32 + 0.5 - 64.0;
            if dx * dx + dy * dy > radius_sq {
                rgba[(y * 128 + x) * 4 + 3] = 0;
            }
        }
    }
    rgba
}

fn minimap_marker_color(dot: &MinimapDot, palette: Option<&[u8]>) -> [u8; 3] {
    let tribe = (dot.tribe_index as usize).min(3);
    let (palette_index, fallback) = match dot.kind {
        MinimapMarkerKind::Person => (
            MINIMAP_PERSON_PALETTE_INDICES[tribe],
            MINIMAP_PERSON_FALLBACK_COLORS[tribe],
        ),
        MinimapMarkerKind::Building => (
            MINIMAP_BUILDING_PALETTE_INDICES[tribe],
            MINIMAP_BUILDING_FALLBACK_COLORS[tribe],
        ),
        MinimapMarkerKind::WildPerson => (
            MINIMAP_WILD_PERSON_PALETTE_INDEX,
            MINIMAP_WILD_PERSON_FALLBACK_COLOR,
        ),
    };
    palette
        .and_then(|entries| {
            let offset = palette_index as usize * 4;
            entries
                .get(offset..offset + 3)
                .map(|rgb| [rgb[0], rgb[1], rgb[2]])
        })
        .unwrap_or(fallback)
}

/// Compute the native panel geometry from Populous' 640×480 virtual canvas.
///
/// PopTB scales x and y independently.  The original status and construction
/// panel entries are data-driven, so keeping those axes separate matters on
/// widescreen captures and keeps the HUD aligned with its native sprites.
pub fn compute_hud_layout(screen_w: f32, screen_h: f32) -> HudLayout {
    use layout::{element_rect, minimap_element, sidebar_width, PANEL_SIDEBAR, PANEL_TAB_PAGE};

    let screen_w_i = screen_w as i32;
    let screen_h_i = screen_h as i32;
    let scale_x = screen_w / layout::VIRTUAL_W as f32;
    let scale_y = screen_h / layout::VIRTUAL_H as f32;
    let sidebar_w = sidebar_width(screen_w_i) as f32;
    let font_scale = (12.0 * scale_y).max(10.0).round();
    let small_font = (font_scale * 0.75).round();
    let mm_pad = 0.0;

    let minimap = element_rect(&PANEL_SIDEBAR, &minimap_element(), screen_w_i, screen_h_i);
    let tab_defs = &layout::SIDEBAR_TABS;
    let construction_tab = element_rect(&PANEL_SIDEBAR, tab_defs[1].1, screen_w_i, screen_h_i);
    let spells_tab = element_rect(&PANEL_SIDEBAR, tab_defs[0].1, screen_w_i, screen_h_i);
    let followers_tab = element_rect(&PANEL_SIDEBAR, tab_defs[2].1, screen_w_i, screen_h_i);
    let mana = element_rect(
        &PANEL_SIDEBAR,
        &layout::SIDEBAR_ELEMENTS[23],
        screen_w_i,
        screen_h_i,
    );
    let info = element_rect(
        &PANEL_SIDEBAR,
        &layout::SIDEBAR_ELEMENTS[22],
        screen_w_i,
        screen_h_i,
    );
    let page = element_rect(
        &PANEL_TAB_PAGE,
        &layout::ElementDef {
            cmd: 0,
            kind: layout::ElementKind::Static,
            ix: 0,
            iy: 0,
            x: 0,
            y: 0,
            w: 100,
            h: 277,
            icon: 0,
            flags: 0,
        },
        screen_w_i,
        screen_h_i,
    );

    let panel_y = page.y as f32;
    let construction_cell_w = 46.0 * scale_x;
    let construction_cell_h = 52.0 * scale_y;
    let line_h = font_scale + 2.0;
    HudLayout {
        screen_w,
        screen_h,
        scale_x,
        scale_y,
        sidebar_w,
        font_scale,
        small_font,
        mm_pad,
        mm_x: minimap.x as f32,
        mm_y: minimap.y as f32,
        mm_size: minimap.w as f32,
        mm_w: minimap.w as f32,
        mm_h: minimap.h as f32,
        mana_bar_y: mana.y as f32,
        mana_bar_h: mana.h as f32,
        pop_y: info.y as f32,
        pop_h: info.h as f32,
        tab_y: construction_tab.y as f32,
        tab_h: construction_tab.h as f32,
        tab_w: construction_tab.w as f32,
        tab_xs: [
            construction_tab.x as f32,
            spells_tab.x as f32,
            followers_tab.x as f32,
        ],
        status_y: mana.y as f32,
        status_h: (info.y + info.h - mana.y) as f32,
        panel_y,
        construction_cell_w,
        construction_cell_h,
        line_h,
    }
}

/// Detect the active construction-tab silhouette.  The other tab artwork is
/// visible for fidelity but intentionally inert in this construction-only UI.
pub fn detect_tab_click(mouse_x: f32, mouse_y: f32, layout: &HudLayout) -> Option<HudTab> {
    let hit_y = 86.0 * layout.scale_y;
    let hit_h = 27.0 * layout.scale_y;
    if mouse_y < hit_y || mouse_y >= hit_y + hit_h {
        return None;
    }
    if mouse_x >= layout.tab_xs[0] && mouse_x < layout.tab_xs[0] + layout.tab_w {
        Some(HudTab::Buildings)
    } else {
        None
    }
}

/// Return the native two-column construction-grid slot under the pointer.
pub fn detect_construction_slot_click(
    mouse_x: f32,
    mouse_y: f32,
    layout: &HudLayout,
) -> Option<usize> {
    let screen_w = layout.screen_w as i32;
    let screen_h = layout.screen_h as i32;
    for (slot, cell) in layout::CONSTRUCTION_PAGE.iter().enumerate() {
        let rect = layout::element_rect(&layout::PANEL_TAB_PAGE, cell, screen_w, screen_h);
        if mouse_x >= rect.x as f32
            && mouse_x < (rect.x + rect.w) as f32
            && mouse_y >= rect.y as f32
            && mouse_y < (rect.y + rect.h) as f32
        {
            return Some(slot);
        }
    }
    None
}

/// Dark inactive in-game tab frame tiles from `hfx0-0.dat`, in nine-patch
/// order `[top-left, top, top-right, left, center, right, bottom-left,
/// bottom, bottom-right]`.
pub const HFX_TAB_FRAME: [u16; 9] = [740, 744, 741, 746, 748, 747, 742, 745, 743];

/// Bright active construction-tab frame. The native HUD screencut shows this
/// raised gold state around the open hut page.
pub const HFX_TAB_FRAME_SELECTED: [u16; 9] = [758, 762, 759, 764, 766, 765, 760, 763, 761];

/// In-game tab silhouettes in visual order: construction, spells, followers.
pub const HFX_TAB_ICONS: [u16; 3] = [676, 678, 680];

/// The selected construction tab keeps the original dark hut silhouette.
/// Sprite 677 is the white hover state, not the idle selected state shown in
/// the native gameplay HUD.
pub const HFX_TAB_ICON_BUILDINGS_SELECTED: u16 = 676;

/// Native rock-arch frame around the minimap; its center stays transparent.
pub const HFX_MINIMAP_FRAME: [u16; 9] = [690, 694, 691, 696, 0, 697, 692, 695, 693];

/// Native shaman status widget in the main sidebar.
pub const HFX_SHAMAN_WIDGET: u16 = 664;

/// Native assets that form the compact in-game status strip.
/// e01's normal (not hover) avatar frame, from callback 0x404130.
pub const HFX_STATUS_AVATAR_FRAME: [u16; 9] = [713, 717, 714, 719, 721, 720, 715, 718, 716];
/// The precomposed 16-bit e01 border retains the original narrow outline and
/// black interior at the status widget's native size.
pub const HFX_STATUS_AVATAR_COMPOSITE: u16 = 469;
/// e12's globe-toggle frame, from callback 0x405c80.
pub const HFX_STATUS_GLOBE_FRAME: [u16; 9] = [767, 771, 768, 773, 775, 774, 769, 772, 770];
/// e12's normal globe state. 876–878 are hover/toggle variants; 875 matches
/// the idle construction HUD in the native capture.
pub const HFX_STATUS_GLOBE: u16 = 875;
/// e19's small help-button frame and e13–18's quick-row cells.
pub const HFX_STATUS_SMALL_FRAME: [u16; 9] = [1005, 1009, 1006, 1011, 1013, 1012, 1007, 1010, 1008];
/// e02's tall status field frame.
pub const HFX_STATUS_TALL_FRAME: [u16; 9] = [1014, 1018, 1015, 1020, 1022, 1021, 1016, 1019, 1017];
pub const HFX_STATUS_BLACK_TEXTURE: u16 = 491;
pub const HFX_STATUS_WHITE_TEXTURE: u16 = 503;
pub const HFX_STATUS_HELP_GLYPH: u16 = 106;
pub const HFX_STATUS_BLUE_CHIP: u16 = 54;
pub const HFX_STATUS_RED_CHIP: u16 = 65;
pub const HFX_STATUS_FOLLOWER_GLYPH: u16 = 666;
pub const HFX_POPULATION_METER: u16 = 1603;

/// The small shaded status-control text comes from `font4-0.dat`, not the
/// fallback HUD bitmap font. FONT4 is indexed from ASCII space, so glyph 41
/// is the native `I` used by the two `II` labels in the reference sidebar.
pub const FONT4_STATUS_GLYPH_I: u16 = 41;
pub const FONT4_HUD_GLYPH_IDS: &[u16] = &[FONT4_STATUS_GLYPH_I];

/// Blue tribe's side-facing idle shaman frame from `HSPR0-0.DAT`.
/// It is the original status-avatar pose used by the reference HUD.
pub const HSPR_STATUS_AVATAR_BLUE: u16 = 6887;

/// In-game construction-button frame tiles, in nine-patch order.  The house
/// tab is rendered by `0x401d10`, whose normal frame table is separate from
/// the spell renderer's table: `popTB.exe` `0x575490`.
pub const HFX_BUILDING_FRAME: [u16; 9] = [821, 825, 822, 827, 829, 828, 823, 826, 824];

/// Native construction-button hover frame (`popTB.exe` `0x5754a8`).
pub const HFX_BUILDING_FRAME_HOVER: [u16; 9] = [839, 843, 840, 845, 847, 846, 841, 844, 842];

/// Native construction-button pressed frame (`popTB.exe` `0x5754c0`).
pub const HFX_BUILDING_FRAME_PRESSED: [u16; 9] = [830, 834, 831, 836, 838, 837, 832, 835, 833];

/// Visual state selected by the original construction-button renderer.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ConstructionButtonState {
    Normal,
    Hovered,
    Pressed,
}

/// Return the exact native nine-patch table for a construction-button state.
pub fn construction_button_frame(state: ConstructionButtonState) -> &'static [u16; 9] {
    match state {
        ConstructionButtonState::Normal => &HFX_BUILDING_FRAME,
        ConstructionButtonState::Hovered => &HFX_BUILDING_FRAME_HOVER,
        ConstructionButtonState::Pressed => &HFX_BUILDING_FRAME_PRESSED,
    }
}

/// Resolve the native visual state for one construction-button cell.  A
/// pressed frame only remains while the pointer is still over the same cell,
/// matching the original button compositor's active-pointer behavior.
pub fn construction_button_state(
    slot: usize,
    hovered_slot: Option<usize>,
    pressed_slot: Option<usize>,
) -> ConstructionButtonState {
    if hovered_slot == Some(slot) && pressed_slot == Some(slot) {
        ConstructionButtonState::Pressed
    } else if hovered_slot == Some(slot) {
        ConstructionButtonState::Hovered
    } else {
        ConstructionButtonState::Normal
    }
}

/// Native HFX icon params from the nine `0x576c20` construction records.
/// `FUN_004018a0` adds 18 while a button is active, selecting the companion
/// highlight family.  The non-sequential third row reflects the original
/// record order; it must not be normalized.
pub const HFX_CONSTRUCTION_ICONS: [u16; 9] = [1028, 1029, 1030, 1032, 1033, 1031, 1034, 1035, 1036];
pub const HFX_CONSTRUCTION_ICONS_HOVER: [u16; 9] =
    [1046, 1047, 1048, 1050, 1051, 1049, 1052, 1053, 1054];

/// Native `?` overlay used by the original blocked construction state.
///
/// `FUN_004018a0` composes this over the regular construction glyph when
/// `FUN_00401c60` reports state 4.  It is deliberately kept separate from
/// the nine building icon families: the overlay is a state signal, not a
/// replacement building icon.
pub const HFX_CONSTRUCTION_BLOCKED_OVERLAY: u16 = 1055;

/// Native construction commands represented by the original house-tab
/// glyphs.  These are command ids, not the `cmd` values used by the panel
/// manager to route clicks between controls.
pub fn construction_command_for_slot(slot: usize) -> Option<u8> {
    layout::CONSTRUCTION_PAGE
        .get(slot)
        .and_then(|element| u8::try_from(element.icon).ok())
}

/// Convert an original construction command id into its bitfield member.
pub const fn construction_command_bit(command: u8) -> u32 {
    if command < u32::BITS as u8 {
        1_u32 << command
    } else {
        0
    }
}

/// Map the building subtype stored in a native level object to the house-tab
/// command that represents it.  The level's Vault object is subtype 18 while
/// its house-tab command is 17, so this mapping must not use the engine's
/// runtime enum directly.
pub const fn construction_command_for_level_building_subtype(subtype: u8) -> Option<u8> {
    match subtype {
        1..=3 => Some(1),
        4 => Some(4),
        5 => Some(5),
        6 => Some(6),
        7 => Some(7),
        8 => Some(8),
        13 => Some(13),
        15 => Some(15),
        18 => Some(17),
        _ => None,
    }
}

/// The three construction-cell outcomes produced by the original HUD setup
/// callback.  A command can be available to the player, visibly blocked
/// because its building occurs on the level, or wholly absent from the panel.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ConstructionSlotAvailability {
    Hidden,
    Available,
    Blocked,
}

impl ConstructionSlotAvailability {
    pub const fn is_interactive(self) -> bool {
        matches!(self, Self::Available)
    }
}

/// Resolve one native construction cell from player capability and
/// level-presence bitfields.  State four in `FUN_00401c60` is represented by
/// `Blocked`; its renderer draws the normal HFX glyph plus sprite 1055.
pub fn construction_slot_availability(
    slot: usize,
    available_commands: u32,
    present_commands: u32,
) -> ConstructionSlotAvailability {
    let Some(command) = construction_command_for_slot(slot) else {
        return ConstructionSlotAvailability::Hidden;
    };
    let command_bit = construction_command_bit(command);
    if available_commands & command_bit != 0 {
        ConstructionSlotAvailability::Available
    } else if present_commands & command_bit != 0 {
        ConstructionSlotAvailability::Blocked
    } else {
        ConstructionSlotAvailability::Hidden
    }
}

/// Resolve the exact construction icon for a native element slot.
pub fn construction_icon_sprite(slot: usize, highlighted: bool) -> Option<u16> {
    let icons = if highlighted {
        &HFX_CONSTRUCTION_ICONS_HOVER
    } else {
        &HFX_CONSTRUCTION_ICONS
    };
    icons.get(slot).copied()
}

/// `GUI_RenderTiledPanel`'s native 16px surface family.  The original UI
/// uses these as four corners, alternating horizontal/vertical edges, and a
/// two-by-two interior cycle instead of stretching or repeating one generic
/// background texture.
///
/// Order: top-left, top-right, bottom-left, bottom-right; top A/B, bottom
/// A/B, left A/B, right A/B; interior AA/AB/BA/BB.
pub const HFX_PANEL_SURFACE_TILES: [u16; 16] = [
    1450, 1451, 1452, 1453, 1454, 1455, 1456, 1457, 1458, 1459, 1460, 1461, 1462, 1463, 1464, 1465,
];

/// Return the interior member of the native tiled-panel's 2×2 cycle.
///
/// `GUI_RenderTiledPanel` at `popTB.exe` `0x4936b0` increments its source
/// row and column counters for the border cells too.  The first interior tile
/// therefore uses the bottom-right member of the four-tile family, not the
/// top-left member.  `row` and `column` here are relative to the first
/// interior cell.
fn panel_surface_interior_tile(tile_ids: &[u16; 16], row: usize, column: usize) -> u16 {
    tile_ids[12 + ((row + 1) & 1) + 2 * ((column + 1) & 1)]
}

/// Verified original HFX art required by the construction HUD.
pub const HFX_HUD_SPRITE_IDS: &[u16] = &[
    HFX_SHAMAN_WIDGET,
    HFX_STATUS_AVATAR_COMPOSITE,
    713,
    714,
    715,
    716,
    717,
    718,
    719,
    720,
    721,
    767,
    768,
    769,
    770,
    771,
    772,
    773,
    774,
    775,
    1005,
    1006,
    1007,
    1008,
    1009,
    1010,
    1011,
    1012,
    1013,
    1014,
    1015,
    1016,
    1017,
    1018,
    1019,
    1020,
    1021,
    1022,
    HFX_STATUS_BLACK_TEXTURE,
    HFX_STATUS_WHITE_TEXTURE,
    HFX_STATUS_GLOBE,
    HFX_STATUS_HELP_GLYPH,
    HFX_STATUS_BLUE_CHIP,
    HFX_STATUS_RED_CHIP,
    HFX_STATUS_FOLLOWER_GLYPH,
    HFX_POPULATION_METER,
    1028,
    1029,
    1030,
    1031,
    1032,
    1033,
    1034,
    1035,
    1036,
    1046,
    1047,
    1048,
    1049,
    1050,
    1051,
    1052,
    1053,
    1054,
    HFX_CONSTRUCTION_BLOCKED_OVERLAY,
    690,
    691,
    692,
    693,
    694,
    695,
    696,
    697,
    740,
    744,
    741,
    746,
    748,
    747,
    742,
    745,
    743,
    758,
    762,
    759,
    764,
    766,
    765,
    760,
    763,
    761,
    821,
    825,
    822,
    827,
    829,
    828,
    823,
    826,
    824,
    839,
    843,
    840,
    845,
    847,
    846,
    841,
    844,
    842,
    830,
    834,
    831,
    836,
    838,
    837,
    832,
    835,
    833,
    676,
    678,
    680,
    1450,
    1451,
    1452,
    1453,
    1454,
    1455,
    1456,
    1457,
    1458,
    1459,
    1460,
    1461,
    1462,
    1463,
    1464,
    1465,
];

/// Verified original HSPR art required by the construction HUD.
pub const HSPR_HUD_SPRITE_IDS: [u16; 1] = [HSPR_STATUS_AVATAR_BLUE];

/// Get the sprite region index for a PSFB panel sprite.
/// Panel sprites are stored after the white pixel (1) + font glyphs (96).
pub fn panel_sprite_index(font_region_start: usize, psfb_index: usize) -> usize {
    font_region_start + 96 + psfb_index
}

// ---------------------------------------------------------------------------
// GPU Renderer
// ---------------------------------------------------------------------------

/// Screen-space 2D sprite/text renderer for the game HUD.
pub struct HudRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: GpuBuffer,
    atlas_bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    atlas_width: u32,
    atlas_height: u32,
    sprite_regions: Vec<SpriteRegion>,
    /// Index of the 1x1 white pixel region for solid rectangles
    white_region_idx: usize,
    /// Index where font glyphs start in sprite_regions
    font_region_start: usize,
    /// Number of sprites loaded from plspanel.spr before the POINT bank.
    panel_sprite_count: usize,
    /// Index where POINT0-0.DAT sprites start in sprite_regions.
    point_region_start: usize,
    /// Atlas regions for the verified in-game HFX UI sprites.
    hfx_regions: HashMap<u16, usize>,
    /// Atlas regions for the verified in-game HSPR status-avatar sprites.
    hspr_regions: HashMap<u16, usize>,
    /// Atlas regions for the native small FONT4 status-control glyphs.
    font4_regions: HashMap<u16, usize>,
    vertices: Vec<HudVertex>,
    /// Number of HUD vertices drawn beneath the separate minimap canvas.
    minimap_split: usize,
    // Minimap texture (updated per-frame)
    minimap_bind_group: Option<wgpu::BindGroup>,
    minimap_texture: Option<GpuTexture>,
}

impl HudRenderer {
    pub const MAX_VERTICES: usize = 65536;

    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        screen_w: f32,
        screen_h: f32,
    ) -> Self {
        // Bind group layout: uniform + texture + sampler
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("hud_bg_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Screen size uniform
        let screen_data = [screen_w, screen_h, 0.0f32, 0.0f32];
        let uniform_buffer =
            GpuBuffer::new_uniform_init(device, bytemuck::bytes_of(&screen_data), "hud_uniforms");

        // Build initial atlas with white pixel + font glyphs (so text works before sprites load)
        let font_rgba = build_font_rgba();
        let font_w = FONT_ATLAS_W;
        let font_h = FONT_ATLAS_H;
        // Atlas layout: white pixel at (0,0), font at (2,0)
        let init_atlas_w = (2 + font_w).next_power_of_two();
        let init_atlas_h = font_h.next_power_of_two();
        let mut init_data = vec![0u8; (init_atlas_w * init_atlas_h * 4) as usize];
        // White pixel at (0,0)
        init_data[0] = 255;
        init_data[1] = 255;
        init_data[2] = 255;
        init_data[3] = 255;
        // Blit font at (2, 0)
        for fy in 0..font_h {
            for fx in 0..font_w {
                let src = ((fy * font_w + fx) * 4) as usize;
                let dst = ((fy * init_atlas_w + 2 + fx) * 4) as usize;
                if dst + 3 < init_data.len() && src + 3 < font_rgba.len() {
                    init_data[dst..dst + 4].copy_from_slice(&font_rgba[src..src + 4]);
                }
            }
        }
        let atlas_tex = GpuTexture::new_2d(
            device,
            queue,
            init_atlas_w,
            init_atlas_h,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            &init_data,
            "hud_atlas_initial",
        );
        let sampler = GpuTexture::create_sampler(device, true);

        let atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("hud_atlas_bg"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&atlas_tex.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // Pipeline
        let shader_source = include_str!("../../../shaders/hud_sprite.wgsl");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("hud_shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("hud_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("hud_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<HudVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 8,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 16,
                            shader_location: 2,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // Vertex buffer (pre-allocated)
        let vb_size = Self::MAX_VERTICES * std::mem::size_of::<HudVertex>();
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hud_vertex_buffer"),
            size: vb_size as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Build sprite regions: white pixel + 96 font glyphs
        let aw = init_atlas_w as f32;
        let ah = init_atlas_h as f32;
        let mut sprite_regions = Vec::new();

        // White pixel region (index 0) — sample center of pixel
        sprite_regions.push(SpriteRegion {
            u0: 0.5 / aw,
            v0: 0.5 / ah,
            u1: 0.5 / aw,
            v1: 0.5 / ah,
            width: 1,
            height: 1,
        });

        // Font glyph regions (indices 1..97)
        let u_step = FONT_GLYPH_W as f32 / aw;
        let v_step = FONT_GLYPH_H as f32 / ah;
        let font_u0 = 2.0 / aw;
        let font_v0 = 0.0;
        for idx in 0..96u32 {
            let col = idx % FONT_COLS;
            let row = idx / FONT_COLS;
            sprite_regions.push(SpriteRegion {
                u0: font_u0 + col as f32 * u_step,
                v0: font_v0 + row as f32 * v_step,
                u1: font_u0 + (col + 1) as f32 * u_step,
                v1: font_v0 + (row + 1) as f32 * v_step,
                width: FONT_GLYPH_W as u16,
                height: FONT_GLYPH_H as u16,
            });
        }

        HudRenderer {
            pipeline,
            vertex_buffer,
            uniform_buffer,
            atlas_bind_group,
            bind_group_layout,
            atlas_width: init_atlas_w,
            atlas_height: init_atlas_h,
            sprite_regions,
            white_region_idx: 0,
            font_region_start: 1,
            panel_sprite_count: 0,
            point_region_start: 97,
            hfx_regions: HashMap::new(),
            hspr_regions: HashMap::new(),
            font4_regions: HashMap::new(),
            vertices: Vec::with_capacity(4096),
            minimap_split: 0,
            minimap_bind_group: None,
            minimap_texture: None,
        }
    }

    /// Build the HUD atlas from the original panel, POINT, HFX, and HSPR banks.
    ///
    /// The panel and POINT banks use separate native palettes.
    pub fn build_atlas(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        panel_sprites: &ContainerPSFB,
        panel_palette: &[u8],
        point_sprites: Option<&ContainerPSFB>,
        point_palette: &[u8],
        hfx_sprites: Option<(&ContainerPSFB, &[u16])>,
        hspr_sprites: Option<(&ContainerPSFB, &[u16])>,
        font4_sprites: Option<(&ContainerPSFB, &[u16])>,
        hfx_palette: &[u8],
        font4_palette: &[u8],
    ) {
        // Phase 1: Convert all sprites to RGBA
        let mut sprite_images: Vec<(u16, u16, Vec<u8>)> = Vec::new(); // (w, h, rgba)
        for i in 0..panel_sprites.len() {
            if let Some(img) = panel_sprites.get_image(i) {
                let w = img.width as u16;
                let h = img.height as u16;
                let rgba = convert_indexed_to_rgba(&img.data, panel_palette, 255);
                sprite_images.push((w, h, rgba));
            } else {
                sprite_images.push((1, 1, vec![0, 0, 0, 0]));
            }
        }
        let panel_sprite_count = sprite_images.len();
        let mut point_sprite_count = 0;
        if let Some(point_sprites) = point_sprites {
            for i in 0..point_sprites.len() {
                if let Some(img) = point_sprites.get_image(i) {
                    let w = img.width as u16;
                    let h = img.height as u16;
                    let rgba = convert_indexed_to_rgba(&img.data, point_palette, 255);
                    sprite_images.push((w, h, rgba));
                } else {
                    sprite_images.push((1, 1, vec![0, 0, 0, 0]));
                }
            }
            point_sprite_count = point_sprites.len();
        }
        let mut hfx_sprite_ids = Vec::new();
        if let Some((hfx_sprites, sprite_ids)) = hfx_sprites {
            for &sprite_id in sprite_ids {
                if let Some(img) = hfx_sprites.get_image(sprite_id as usize) {
                    let w = img.width as u16;
                    let h = img.height as u16;
                    let rgba = convert_indexed_to_rgba(&img.data, hfx_palette, 255);
                    sprite_images.push((w, h, rgba));
                    hfx_sprite_ids.push(sprite_id);
                }
            }
        }
        let mut hspr_sprite_ids = Vec::new();
        if let Some((hspr_sprites, sprite_ids)) = hspr_sprites {
            for &sprite_id in sprite_ids {
                if let Some(img) = hspr_sprites.get_image(sprite_id as usize) {
                    let w = img.width as u16;
                    let h = img.height as u16;
                    let rgba = convert_indexed_to_rgba(&img.data, hfx_palette, 255);
                    sprite_images.push((w, h, rgba));
                    hspr_sprite_ids.push(sprite_id);
                }
            }
        }
        let mut font4_sprite_ids = Vec::new();
        if let Some((font4_sprites, sprite_ids)) = font4_sprites {
            for &sprite_id in sprite_ids {
                if let Some(img) = font4_sprites.get_image(sprite_id as usize) {
                    let w = img.width as u16;
                    let h = img.height as u16;
                    let rgba = convert_indexed_to_rgba(&img.data, font4_palette, 255);
                    sprite_images.push((w, h, rgba));
                    font4_sprite_ids.push(sprite_id);
                }
            }
        }

        // Phase 2: Calculate atlas dimensions using shelf packing
        let font_w = FONT_ATLAS_W as u16;
        let font_h = FONT_ATLAS_H as u16;
        let atlas_w: u32 = 1024;

        // Pack all items: white pixel (1x1), font atlas, then sprite images
        let mut all_items: Vec<(u16, u16)> = Vec::with_capacity(2 + sprite_images.len());
        all_items.push((1, 1)); // white pixel
        all_items.push((font_w, font_h)); // font atlas
        for (w, h, _) in &sprite_images {
            all_items.push((*w, *h));
        }
        let (all_placements, atlas_h) = shelf_pack(&all_items, atlas_w);
        let atlas_w = atlas_w.next_power_of_two();

        // Extract placements
        let font_placement_x = all_placements[1].0;
        let font_placement_y = all_placements[1].1;
        // Sprite placements start at index 2
        let placements: Vec<(u32, u32)> = all_placements[2..].to_vec();

        // Phase 3: Blit into atlas
        let mut atlas_data = vec![0u8; (atlas_w * atlas_h * 4) as usize];

        // Blit white pixel
        let (wp_x, wp_y) = all_placements[0];
        let wp = ((wp_y * atlas_w + wp_x) * 4) as usize;
        atlas_data[wp] = 255;
        atlas_data[wp + 1] = 255;
        atlas_data[wp + 2] = 255;
        atlas_data[wp + 3] = 255;

        // Blit font atlas
        let font_atlas_rgba = build_font_rgba();
        for fy in 0..font_h as u32 {
            for fx in 0..font_w as u32 {
                let src = ((fy * font_w as u32 + fx) * 4) as usize;
                let dst =
                    (((font_placement_y + fy) * atlas_w + font_placement_x + fx) * 4) as usize;
                if dst + 3 < atlas_data.len() && src + 3 < font_atlas_rgba.len() {
                    atlas_data[dst..dst + 4].copy_from_slice(&font_atlas_rgba[src..src + 4]);
                }
            }
        }

        // Blit sprite images
        for (i, (w, h, rgba)) in sprite_images.iter().enumerate() {
            let (px, py) = placements[i];
            for sy in 0..*h as u32 {
                for sx in 0..*w as u32 {
                    let src = ((sy * *w as u32 + sx) * 4) as usize;
                    let dst = (((py + sy) * atlas_w + px + sx) * 4) as usize;
                    if dst + 3 < atlas_data.len() && src + 3 < rgba.len() {
                        atlas_data[dst..dst + 4].copy_from_slice(&rgba[src..src + 4]);
                    }
                }
            }
        }

        // Phase 4: Build sprite regions
        let aw = atlas_w as f32;
        let ah = atlas_h as f32;
        let mut regions = Vec::new();

        // White pixel region (index 0)
        regions.push(SpriteRegion {
            u0: 0.5 / aw,
            v0: 0.5 / ah,
            u1: 0.5 / aw,
            v1: 0.5 / ah,
            width: 1,
            height: 1,
        });

        // Font glyph regions (indices 1..96)
        let font_start = regions.len();
        let u_step = FONT_GLYPH_W as f32 / aw;
        let v_step = FONT_GLYPH_H as f32 / ah;
        let font_u0 = font_placement_x as f32 / aw;
        let font_v0 = font_placement_y as f32 / ah;
        for idx in 0..96u32 {
            let col = idx % FONT_COLS;
            let row = idx / FONT_COLS;
            regions.push(SpriteRegion {
                u0: font_u0 + col as f32 * u_step,
                v0: font_v0 + row as f32 * v_step,
                u1: font_u0 + (col + 1) as f32 * u_step,
                v1: font_v0 + (row + 1) as f32 * v_step,
                width: FONT_GLYPH_W as u16,
                height: FONT_GLYPH_H as u16,
            });
        }

        // Panel sprite regions
        for (i, (w, h, _)) in sprite_images.iter().enumerate() {
            let (px, py) = placements[i];
            regions.push(SpriteRegion {
                u0: px as f32 / aw,
                v0: py as f32 / ah,
                u1: (px + *w as u32) as f32 / aw,
                v1: (py + *h as u32) as f32 / ah,
                width: *w,
                height: *h,
            });
        }

        // Phase 5: Upload atlas
        let atlas_tex = GpuTexture::new_2d(
            device,
            queue,
            atlas_w,
            atlas_h,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            &atlas_data,
            "hud_atlas",
        );
        let sampler = GpuTexture::create_sampler(device, true);

        self.atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("hud_atlas_bg"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&atlas_tex.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        self.atlas_width = atlas_w;
        self.atlas_height = atlas_h;
        self.sprite_regions = regions;
        self.white_region_idx = 0;
        self.font_region_start = font_start;
        self.panel_sprite_count = panel_sprite_count;
        self.point_region_start = font_start + 96 + panel_sprite_count;
        self.hfx_regions.clear();
        self.hspr_regions.clear();
        self.font4_regions.clear();
        let hfx_region_start = self.point_region_start + point_sprite_count;
        for (offset, sprite_id) in hfx_sprite_ids.iter().enumerate() {
            self.hfx_regions
                .insert(*sprite_id, hfx_region_start + offset);
        }
        let hspr_region_start = hfx_region_start + hfx_sprite_ids.len();
        for (offset, sprite_id) in hspr_sprite_ids.iter().enumerate() {
            self.hspr_regions
                .insert(*sprite_id, hspr_region_start + offset);
        }
        let font4_region_start = hspr_region_start + hspr_sprite_ids.len();
        for (offset, sprite_id) in font4_sprite_ids.iter().enumerate() {
            self.font4_regions
                .insert(*sprite_id, font4_region_start + offset);
        }

        log::info!(
            "[hud] Atlas built: {}x{}, {} sprites, {} font glyphs, {} total regions",
            atlas_w,
            atlas_h,
            sprite_images.len(),
            96,
            self.sprite_regions.len()
        );
    }

    pub fn begin_frame(&mut self) {
        self.vertices.clear();
        self.minimap_split = 0;
    }

    /// Subsequent HUD vertices render above the minimap canvas. This mirrors
    /// the original panel compositor: repeated sidebar art first, minimap,
    /// then its frame and all controls.
    pub fn mark_minimap_split(&mut self) {
        self.minimap_split = self.vertices.len();
    }

    pub fn push_quad(
        &mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        u0: f32,
        v0: f32,
        u1: f32,
        v1: f32,
        color: [f32; 4],
    ) {
        self.vertices.extend_from_slice(&generate_quad_vertices(
            x0, y0, x1, y1, u0, v0, u1, v1, color,
        ));
    }

    /// Draw a solid colored rectangle.
    pub fn draw_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
        let r = &self.sprite_regions[self.white_region_idx];
        self.push_quad(x, y, x + w, y + h, r.u0, r.v0, r.u1, r.v1, color);
    }

    /// Draw a solid triangle using the atlas' white pixel.
    pub fn draw_triangle(&mut self, points: [[f32; 2]; 3], color: [f32; 4]) {
        let r = &self.sprite_regions[self.white_region_idx];
        let u = (r.u0 + r.u1) * 0.5;
        let v = (r.v0 + r.v1) * 0.5;
        self.vertices
            .extend(points.into_iter().map(|position| HudVertex {
                position,
                uv: [u, v],
                color,
            }));
    }

    /// Draw a solid line segment with stable pixel thickness.
    pub fn draw_line(&mut self, from: [f32; 2], to: [f32; 2], thickness: f32, color: [f32; 4]) {
        let dx = to[0] - from[0];
        let dy = to[1] - from[1];
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.001 {
            return;
        }
        let nx = -dy / len * thickness * 0.5;
        let ny = dx / len * thickness * 0.5;
        let a = [from[0] + nx, from[1] + ny];
        let b = [from[0] - nx, from[1] - ny];
        let c = [to[0] - nx, to[1] - ny];
        let d = [to[0] + nx, to[1] + ny];
        self.draw_triangle([a, b, c], color);
        self.draw_triangle([a, c, d], color);
    }

    /// Draw a sprite from the atlas at screen position (x, y) with scale.
    pub fn draw_sprite(&mut self, sprite_idx: usize, x: f32, y: f32, scale_x: f32, scale_y: f32) {
        self.draw_sprite_tinted(sprite_idx, x, y, scale_x, scale_y, [1.0; 4]);
    }

    /// Draw an atlas sprite with a color multiplier.
    pub fn draw_sprite_tinted(
        &mut self,
        sprite_idx: usize,
        x: f32,
        y: f32,
        scale_x: f32,
        scale_y: f32,
        color: [f32; 4],
    ) {
        if sprite_idx >= self.sprite_regions.len() {
            return;
        }
        let r = self.sprite_regions[sprite_idx].clone();
        let w = r.width as f32 * scale_x;
        let h = r.height as f32 * scale_y;
        self.push_quad(x, y, x + w, y + h, r.u0, r.v0, r.u1, r.v1, color);
    }

    /// Draw text using the embedded bitmap font.
    pub fn draw_text(&mut self, text: &str, x0: f32, y0: f32, scale: f32, color: [f32; 4]) {
        let mut cx = x0;
        let mut cy = y0;
        for ch in text.chars() {
            if ch == '\n' {
                cx = x0;
                cy += scale;
                continue;
            }
            let code = ch as u32;
            if code < 32 || code > 126 {
                cx += scale;
                continue;
            }
            let glyph_idx = (code - 32) as usize;
            let region_idx = self.font_region_start + glyph_idx;
            if region_idx < self.sprite_regions.len() {
                let r = self.sprite_regions[region_idx].clone();
                self.push_quad(
                    cx,
                    cy,
                    cx + scale,
                    cy + scale,
                    r.u0,
                    r.v0,
                    r.u1,
                    r.v1,
                    color,
                );
            }
            cx += scale;
        }
    }

    /// Draw text at one of three sizes using FontData scaling.
    /// font_scale: 1 = 8px (small/tooltip), 2 = 16px (standard HUD), 3 = 24px (headings).
    /// The `px_size` parameter controls how large each glyph renders on screen (in pixels).
    /// For default behavior matching the font_scale: pass `font_scale * 8.0`.
    pub fn draw_text_sized(
        &mut self,
        text: &str,
        x0: f32,
        y0: f32,
        font_scale: u32,
        color: [f32; 4],
    ) {
        let px_size = (font_scale * FONT_GLYPH_W) as f32;
        self.draw_text(text, x0, y0, px_size, color);
    }

    /// Get the sprite region index for panel sprites (offset past white pixel + font glyphs).
    pub fn panel_sprite_index(&self, psfb_index: usize) -> usize {
        panel_sprite_index(self.font_region_start, psfb_index)
    }

    /// Get the sprite region index for POINT0-0.DAT sprites.
    pub fn point_sprite_index(&self, psfb_index: usize) -> usize {
        self.point_region_start + psfb_index
    }

    pub fn sprite_size(&self, sprite_idx: usize) -> Option<(u16, u16)> {
        self.sprite_regions
            .get(sprite_idx)
            .map(|region| (region.width, region.height))
    }

    /// Native pixel dimensions of a verified HFX UI sprite.
    pub fn hfx_size(&self, sprite_id: u16) -> Option<(u16, u16)> {
        self.hfx_regions
            .get(&sprite_id)
            .and_then(|&sprite_idx| self.sprite_size(sprite_idx))
    }

    /// Native pixel dimensions of a verified HSPR HUD sprite.
    pub fn hspr_size(&self, sprite_id: u16) -> Option<(u16, u16)> {
        self.hspr_regions
            .get(&sprite_id)
            .and_then(|&sprite_idx| self.sprite_size(sprite_idx))
    }

    /// Native pixel dimensions of a small FONT4 status-control glyph.
    pub fn font4_size(&self, sprite_id: u16) -> Option<(u16, u16)> {
        self.font4_regions
            .get(&sprite_id)
            .and_then(|&sprite_idx| self.sprite_size(sprite_idx))
    }

    /// Draw a verified HFX UI sprite at native size times `scale`.
    pub fn draw_hfx(&mut self, sprite_id: u16, x: f32, y: f32, scale: f32) -> bool {
        self.draw_hfx_scaled(sprite_id, x, y, scale, scale)
    }

    /// Draw a verified HFX UI sprite with the original panel's independent
    /// horizontal and vertical scales.
    pub fn draw_hfx_scaled(
        &mut self,
        sprite_id: u16,
        x: f32,
        y: f32,
        scale_x: f32,
        scale_y: f32,
    ) -> bool {
        let Some(&sprite_idx) = self.hfx_regions.get(&sprite_id) else {
            return false;
        };
        self.draw_sprite(sprite_idx, x, y, scale_x, scale_y);
        true
    }

    /// Draw a verified HFX UI sprite stretched to an exact rectangle.
    pub fn draw_hfx_stretched(
        &mut self,
        sprite_id: u16,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> bool {
        let Some(&sprite_idx) = self.hfx_regions.get(&sprite_id) else {
            return false;
        };
        let region = self.sprite_regions[sprite_idx].clone();
        self.push_quad(
            x,
            y,
            x + width,
            y + height,
            region.u0,
            region.v0,
            region.u1,
            region.v1,
            [1.0; 4],
        );
        true
    }

    /// Draw a verified HFX UI sprite mirrored horizontally. The original
    /// population meter fills from the right edge in the construction HUD.
    pub fn draw_hfx_flipped(&mut self, sprite_id: u16, x: f32, y: f32, scale: f32) -> bool {
        self.draw_hfx_flipped_scaled(sprite_id, x, y, scale, scale)
    }

    /// Draw a verified HFX UI sprite mirrored horizontally with independent
    /// panel-axis scales.
    pub fn draw_hfx_flipped_scaled(
        &mut self,
        sprite_id: u16,
        x: f32,
        y: f32,
        scale_x: f32,
        scale_y: f32,
    ) -> bool {
        let Some(&sprite_idx) = self.hfx_regions.get(&sprite_id) else {
            return false;
        };
        let region = self.sprite_regions[sprite_idx].clone();
        self.push_quad(
            x,
            y,
            x + region.width as f32 * scale_x,
            y + region.height as f32 * scale_y,
            region.u1,
            region.v0,
            region.u0,
            region.v1,
            [1.0; 4],
        );
        true
    }

    /// Draw a verified HSPR status-avatar sprite at native size times `scale`.
    pub fn draw_hspr(&mut self, sprite_id: u16, x: f32, y: f32, scale: f32) -> bool {
        self.draw_hspr_scaled(sprite_id, x, y, scale, scale)
    }

    /// Draw a verified HSPR sprite with independent panel-axis scales.
    pub fn draw_hspr_scaled(
        &mut self,
        sprite_id: u16,
        x: f32,
        y: f32,
        scale_x: f32,
        scale_y: f32,
    ) -> bool {
        let Some(&sprite_idx) = self.hspr_regions.get(&sprite_id) else {
            return false;
        };
        self.draw_sprite(sprite_idx, x, y, scale_x, scale_y);
        true
    }

    /// Draw a native FONT4 glyph with PopTB's independent panel-axis scale.
    pub fn draw_font4_scaled(
        &mut self,
        sprite_id: u16,
        x: f32,
        y: f32,
        scale_x: f32,
        scale_y: f32,
    ) -> bool {
        let Some(&sprite_idx) = self.font4_regions.get(&sprite_id) else {
            return false;
        };
        self.draw_sprite(sprite_idx, x, y, scale_x, scale_y);
        true
    }

    /// Repeat an original HFX texture at native pixel size, clipping the last
    /// row and column instead of stretching its texels.
    pub fn draw_hfx_tiled(
        &mut self,
        sprite_id: u16,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        scale: f32,
    ) -> bool {
        self.draw_hfx_tiled_scaled(sprite_id, x, y, width, height, scale, scale)
    }

    /// Repeat original HFX texture art using PopTB's independent x/y panel
    /// scaling, clipping partial edge tiles without filtering replacement art.
    pub fn draw_hfx_tiled_scaled(
        &mut self,
        sprite_id: u16,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        scale_x: f32,
        scale_y: f32,
    ) -> bool {
        let Some(&sprite_idx) = self.hfx_regions.get(&sprite_id) else {
            return false;
        };
        let region = self.sprite_regions[sprite_idx].clone();
        let tile_w = region.width as f32 * scale_x;
        let tile_h = region.height as f32 * scale_y;
        if tile_w <= 0.0 || tile_h <= 0.0 || width <= 0.0 || height <= 0.0 {
            return false;
        }

        let mut tile_y = y;
        while tile_y < y + height {
            let draw_h = tile_h.min(y + height - tile_y);
            let v1 = region.v0 + (region.v1 - region.v0) * (draw_h / tile_h);
            let mut tile_x = x;
            while tile_x < x + width {
                let draw_w = tile_w.min(x + width - tile_x);
                let u1 = region.u0 + (region.u1 - region.u0) * (draw_w / tile_w);
                self.push_quad(
                    tile_x,
                    tile_y,
                    tile_x + draw_w,
                    tile_y + draw_h,
                    region.u0,
                    region.v0,
                    u1,
                    v1,
                    [1.0; 4],
                );
                tile_x += tile_w;
            }
            tile_y += tile_h;
        }
        true
    }

    /// Compose a native `GUI_RenderTiledPanel` surface.  Unlike a nine-patch,
    /// PopTB keeps the source texels at their native size and cycles the edge
    /// and interior variants as the panel grows.  The final row/column is
    /// source-clipped so a 100-unit sidebar never leaks a tile into the 3D
    /// viewport.
    pub fn draw_hfx_panel_surface_scaled(
        &mut self,
        tile_ids: &[u16; 16],
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        scale_x: f32,
        scale_y: f32,
    ) -> bool {
        let Some((tile_w, tile_h)) = self.hfx_size(tile_ids[0]) else {
            return false;
        };
        if width <= 0.0 || height <= 0.0 || scale_x <= 0.0 || scale_y <= 0.0 {
            return false;
        }
        let tile_w = tile_w as f32 * scale_x;
        let tile_h = tile_h as f32 * scale_y;
        let corner_w = tile_w.min(width * 0.5);
        let corner_h = tile_h.min(height * 0.5);
        let right = x + width;
        let bottom = y + height;
        let inner_left = x + corner_w;
        let inner_right = right - corner_w;
        let inner_top = y + corner_h;
        let inner_bottom = bottom - corner_h;

        // Corners: top-left, top-right, bottom-left, bottom-right.
        self.draw_hfx_clipped_scaled(tile_ids[0], x, y, corner_w, corner_h, scale_x, scale_y);
        self.draw_hfx_clipped_scaled(
            tile_ids[1],
            inner_right,
            y,
            corner_w,
            corner_h,
            scale_x,
            scale_y,
        );
        self.draw_hfx_clipped_scaled(
            tile_ids[2],
            x,
            inner_bottom,
            corner_w,
            corner_h,
            scale_x,
            scale_y,
        );
        self.draw_hfx_clipped_scaled(
            tile_ids[3],
            inner_right,
            inner_bottom,
            corner_w,
            corner_h,
            scale_x,
            scale_y,
        );

        let mut column = 0usize;
        let mut tile_x = inner_left;
        while tile_x < inner_right {
            let draw_w = tile_w.min(inner_right - tile_x);
            self.draw_hfx_clipped_scaled(
                tile_ids[4 + column % 2],
                tile_x,
                y,
                draw_w,
                corner_h,
                scale_x,
                scale_y,
            );
            self.draw_hfx_clipped_scaled(
                tile_ids[6 + column % 2],
                tile_x,
                inner_bottom,
                draw_w,
                corner_h,
                scale_x,
                scale_y,
            );
            column += 1;
            tile_x += tile_w;
        }

        let mut row = 0usize;
        let mut tile_y = inner_top;
        while tile_y < inner_bottom {
            let draw_h = tile_h.min(inner_bottom - tile_y);
            self.draw_hfx_clipped_scaled(
                tile_ids[8 + row % 2],
                x,
                tile_y,
                corner_w,
                draw_h,
                scale_x,
                scale_y,
            );
            self.draw_hfx_clipped_scaled(
                tile_ids[10 + row % 2],
                inner_right,
                tile_y,
                corner_w,
                draw_h,
                scale_x,
                scale_y,
            );

            let mut column = 0usize;
            let mut tile_x = inner_left;
            while tile_x < inner_right {
                let draw_w = tile_w.min(inner_right - tile_x);
                // The original loop counts the top/left border as tile zero,
                // so the visible interior cycle is offset by one in both
                // axes.  Keep that phase rather than starting a new pattern
                // at the panel's inner corner.
                let interior = panel_surface_interior_tile(tile_ids, row, column);
                self.draw_hfx_clipped_scaled(
                    interior, tile_x, tile_y, draw_w, draw_h, scale_x, scale_y,
                );
                column += 1;
                tile_x += tile_w;
            }
            row += 1;
            tile_y += tile_h;
        }
        true
    }

    fn draw_hfx_clipped_scaled(
        &mut self,
        sprite_id: u16,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        scale_x: f32,
        scale_y: f32,
    ) -> bool {
        let Some(&sprite_idx) = self.hfx_regions.get(&sprite_id) else {
            return false;
        };
        let region = self.sprite_regions[sprite_idx].clone();
        let native_w = region.width as f32 * scale_x;
        let native_h = region.height as f32 * scale_y;
        if native_w <= 0.0 || native_h <= 0.0 || width <= 0.0 || height <= 0.0 {
            return false;
        }
        let draw_w = width.min(native_w);
        let draw_h = height.min(native_h);
        let u1 = region.u0 + (region.u1 - region.u0) * (draw_w / native_w);
        let v1 = region.v0 + (region.v1 - region.v0) * (draw_h / native_h);
        self.push_quad(
            x,
            y,
            x + draw_w,
            y + draw_h,
            region.u0,
            region.v0,
            u1,
            v1,
            [1.0; 4],
        );
        true
    }

    /// Draw only the border of an original nine-patch, preserving the native
    /// repeated panel texture underneath its transparent-looking center.
    pub fn draw_hfx_nine_patch_border(
        &mut self,
        sprite_ids: &[u16; 9],
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        scale: f32,
    ) -> bool {
        self.draw_hfx_nine_patch_border_scaled(sprite_ids, x, y, width, height, scale, scale)
    }

    /// Draw just a native nine-patch border using independent x/y scales.
    pub fn draw_hfx_nine_patch_border_scaled(
        &mut self,
        sprite_ids: &[u16; 9],
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        scale_x: f32,
        scale_y: f32,
    ) -> bool {
        let mut border = *sprite_ids;
        border[4] = 0;
        self.draw_hfx_nine_patch_scaled(&border, x, y, width, height, scale_x, scale_y)
    }

    /// Draw one of the original HFX nine-patch widget frames.
    pub fn draw_hfx_nine_patch(
        &mut self,
        sprite_ids: &[u16; 9],
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        scale: f32,
    ) -> bool {
        self.draw_hfx_nine_patch_scaled(sprite_ids, x, y, width, height, scale, scale)
    }

    /// Draw one original HFX nine-patch at an independently scaled panel
    /// rectangle.  This matches the binary's separate 640-wide/480-high
    /// virtual-coordinate conversion.
    pub fn draw_hfx_nine_patch_scaled(
        &mut self,
        sprite_ids: &[u16; 9],
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        scale_x: f32,
        scale_y: f32,
    ) -> bool {
        let Some((corner_w, corner_h)) = self.hfx_size(sprite_ids[0]) else {
            return false;
        };
        let corner_w = (corner_w as f32 * scale_x).min(width * 0.5);
        let corner_h = (corner_h as f32 * scale_y).min(height * 0.5);
        let x1 = x + corner_w;
        let y1 = y + corner_h;
        let x2 = x + width - corner_w;
        let y2 = y + height - corner_h;
        let cells = [
            (sprite_ids[0], x, y, corner_w, corner_h),
            (sprite_ids[1], x1, y, x2 - x1, corner_h),
            (sprite_ids[2], x2, y, corner_w, corner_h),
            (sprite_ids[3], x, y1, corner_w, y2 - y1),
            (sprite_ids[4], x1, y1, x2 - x1, y2 - y1),
            (sprite_ids[5], x2, y1, corner_w, y2 - y1),
            (sprite_ids[6], x, y2, corner_w, corner_h),
            (sprite_ids[7], x1, y2, x2 - x1, corner_h),
            (sprite_ids[8], x2, y2, corner_w, corner_h),
        ];
        for (sprite_id, cell_x, cell_y, cell_w, cell_h) in cells {
            if sprite_id != 0 && cell_w > 0.0 && cell_h > 0.0 {
                self.draw_hfx_stretched(sprite_id, cell_x, cell_y, cell_w, cell_h);
            }
        }
        true
    }

    /// Update the minimap texture from pre-built MinimapData.
    pub fn update_minimap(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &MinimapData,
    ) {
        let rgba = generate_minimap_rgba(data);

        let tex = GpuTexture::new_2d(
            device,
            queue,
            128,
            128,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            &rgba,
            "minimap",
        );
        let sampler = GpuTexture::create_sampler(device, false);

        self.minimap_bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("minimap_bg"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&tex.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        }));
        self.minimap_texture = Some(tex);
    }

    /// Render the HUD. Issues draw calls with the atlas bind group, and optionally the minimap bind group.
    pub fn render_full(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        queue: &wgpu::Queue,
        screen_w: f32,
        screen_h: f32,
        minimap_rect: Option<(f32, f32, f32, f32)>,
    ) {
        if self.vertices.is_empty() && self.minimap_bind_group.is_none() {
            return;
        }

        // Update screen size uniform
        let screen_data = [screen_w, screen_h, 0.0f32, 0.0f32];
        self.uniform_buffer
            .update(queue, 0, bytemuck::bytes_of(&screen_data));

        // Upload vertex data
        let data: &[u8] = bytemuck::cast_slice(&self.vertices);
        queue.write_buffer(&self.vertex_buffer, 0, data);

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("hud_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });
        pass.set_pipeline(&self.pipeline);

        // Sidebar textures are laid down beneath the minimap canvas.
        let minimap_split = self.minimap_split.min(self.vertices.len());
        if minimap_split > 0 {
            pass.set_bind_group(0, &self.atlas_bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.draw(0..minimap_split as u32, 0..1);
        }

        // Draw the minimap between the background and the HUD controls.
        if let (Some(ref mm_bg), Some((mx, my, mw, mh))) = (&self.minimap_bind_group, minimap_rect)
        {
            // Build minimap quad inline (6 vertices at the very start)
            let mm_verts = [
                HudVertex {
                    position: [mx, my],
                    uv: [0.0, 0.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                },
                HudVertex {
                    position: [mx + mw, my],
                    uv: [1.0, 0.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                },
                HudVertex {
                    position: [mx, my + mh],
                    uv: [0.0, 1.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                },
                HudVertex {
                    position: [mx, my + mh],
                    uv: [0.0, 1.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                },
                HudVertex {
                    position: [mx + mw, my],
                    uv: [1.0, 0.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                },
                HudVertex {
                    position: [mx + mw, my + mh],
                    uv: [1.0, 1.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                },
            ];
            let mm_data: &[u8] = bytemuck::cast_slice(&mm_verts);
            // Write minimap vertices at the end of the existing vertex data
            let mm_offset = (self.vertices.len() * std::mem::size_of::<HudVertex>()) as u64;
            queue.write_buffer(&self.vertex_buffer, mm_offset, mm_data);

            pass.set_bind_group(0, mm_bg, &[]);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.draw(
                self.vertices.len() as u32..self.vertices.len() as u32 + 6,
                0..1,
            );
        }

        // Draw all controls and borders on top of the minimap.
        if minimap_split < self.vertices.len() {
            pass.set_bind_group(0, &self.atlas_bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.draw(minimap_split as u32..self.vertices.len() as u32, 0..1);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- build_font_rgba --

    #[test]
    fn build_font_rgba_correct_size() {
        // Arrange: expected size is 128 * 48 * 4 = 24576 bytes
        let expected_len = (FONT_ATLAS_W * FONT_ATLAS_H * 4) as usize;

        // Act
        let rgba = build_font_rgba();

        // Assert
        assert_eq!(rgba.len(), expected_len);
    }

    #[test]
    fn build_font_rgba_known_set_pixel() {
        // Arrange: 'A' is glyph index 33 (ASCII 65 - 32).
        // FONT_8X8[33] row 0 = 0x38 = 0b00111000, so pixel (2,0) relative to glyph is set.
        // Glyph 33: col = 33 % 16 = 1, row = 33 / 16 = 2
        // Atlas pixel = (1*8 + 2, 2*8 + 0) = (10, 16)
        let px = 1 * FONT_GLYPH_W + 2;
        let py = 2 * FONT_GLYPH_H + 0;
        let off = ((py * FONT_ATLAS_W + px) * 4) as usize;

        // Act
        let rgba = build_font_rgba();

        // Assert: white opaque pixel
        assert_eq!(rgba[off], 255);
        assert_eq!(rgba[off + 1], 255);
        assert_eq!(rgba[off + 2], 255);
        assert_eq!(rgba[off + 3], 255);
    }

    #[test]
    fn build_font_rgba_space_glyph_empty() {
        // Arrange: space (glyph 0) is all zeros, so (0,0) in the atlas should be transparent
        let off = 0usize; // pixel (0, 0) = glyph 0, row 0

        // Act
        let rgba = build_font_rgba();

        // Assert
        assert_eq!(rgba[off + 3], 0); // alpha = 0
    }

    #[test]
    fn build_font_rgba_all_opaque_pixels_are_white() {
        // Arrange & Act
        let rgba = build_font_rgba();

        // Assert: every pixel with alpha > 0 must be pure white
        for i in (0..rgba.len()).step_by(4) {
            if rgba[i + 3] > 0 {
                assert_eq!(rgba[i], 255, "R at offset {}", i);
                assert_eq!(rgba[i + 1], 255, "G at offset {}", i);
                assert_eq!(rgba[i + 2], 255, "B at offset {}", i);
            }
        }
    }

    // -- generate_quad_vertices --

    #[test]
    fn generate_quad_vertices_returns_six() {
        // Arrange
        let color = [1.0, 1.0, 1.0, 1.0];

        // Act
        let verts = generate_quad_vertices(0.0, 0.0, 10.0, 10.0, 0.0, 0.0, 1.0, 1.0, color);

        // Assert
        assert_eq!(verts.len(), 6);
    }

    #[test]
    fn generate_quad_vertices_triangle_winding() {
        // Arrange
        let color = [1.0, 0.0, 0.0, 1.0];

        // Act
        let v = generate_quad_vertices(10.0, 20.0, 50.0, 80.0, 0.1, 0.2, 0.9, 0.8, color);

        // Assert: first triangle = TL, TR, BL
        assert_eq!(v[0].position, [10.0, 20.0]); // TL
        assert_eq!(v[1].position, [50.0, 20.0]); // TR
        assert_eq!(v[2].position, [10.0, 80.0]); // BL
                                                 // Second triangle = BL, TR, BR
        assert_eq!(v[3].position, [10.0, 80.0]); // BL
        assert_eq!(v[4].position, [50.0, 20.0]); // TR
        assert_eq!(v[5].position, [50.0, 80.0]); // BR
    }

    #[test]
    fn generate_quad_vertices_uv_matches_input() {
        // Arrange
        let color = [1.0; 4];

        // Act
        let v = generate_quad_vertices(0.0, 0.0, 1.0, 1.0, 0.25, 0.5, 0.75, 1.0, color);

        // Assert
        assert_eq!(v[0].uv, [0.25, 0.5]); // TL
        assert_eq!(v[1].uv, [0.75, 0.5]); // TR
        assert_eq!(v[5].uv, [0.75, 1.0]); // BR
    }

    // -- shelf_pack --

    #[test]
    fn shelf_pack_single_item() {
        // Arrange
        let items = [(32, 32)];

        // Act
        let (placements, height) = shelf_pack(&items, 1024);

        // Assert
        assert_eq!(placements[0], (0, 0));
        assert!(height >= 32);
    }

    #[test]
    fn shelf_pack_items_fit_one_row() {
        // Arrange: 3 items of 100px wide, atlas = 1024 (plenty of room)
        let items = [(100, 20), (100, 30), (100, 25)];

        // Act
        let (placements, _height) = shelf_pack(&items, 1024);

        // Assert: all on row y=0
        assert_eq!(placements[0].1, 0);
        assert_eq!(placements[1].1, 0);
        assert_eq!(placements[2].1, 0);
        // x positions increase
        assert!(placements[1].0 > placements[0].0);
        assert!(placements[2].0 > placements[1].0);
    }

    #[test]
    fn shelf_pack_overflow_wraps_to_next_shelf() {
        // Arrange: atlas width 100, items of 60px each — second won't fit on first row
        let items = [(60, 20), (60, 30)];

        // Act
        let (placements, _height) = shelf_pack(&items, 100);

        // Assert: first item at y=0, second wraps to y=20
        assert_eq!(placements[0], (0, 0));
        assert_eq!(placements[1].0, 0);
        assert_eq!(placements[1].1, 20); // shelf_h from first row
    }

    #[test]
    fn shelf_pack_height_is_power_of_two() {
        // Arrange
        let items = [(50, 30), (50, 40), (50, 20)];

        // Act
        let (_placements, height) = shelf_pack(&items, 60);

        // Assert: height is power of two
        assert!(height.is_power_of_two());
        // Must be at least as tall as content
        assert!(height >= 30 + 40); // two rows: 40 + 20
    }

    // -- convert_indexed_to_rgba --

    #[test]
    fn convert_indexed_opaque_pixel() {
        // Arrange: palette entry 5 = RGBX (10, 20, 30, 255)
        let mut palette = vec![0u8; 256 * 4];
        palette[5 * 4] = 10; // R
        palette[5 * 4 + 1] = 20; // G
        palette[5 * 4 + 2] = 30; // B
        palette[5 * 4 + 3] = 255;
        let indexed = [5u8];

        // Act
        let rgba = convert_indexed_to_rgba(&indexed, &palette, 255);

        // Assert: original RGB channel order is retained
        assert_eq!(rgba[0], 10); // R
        assert_eq!(rgba[1], 20); // G
        assert_eq!(rgba[2], 30); // B
        assert_eq!(rgba[3], 255); // A
    }

    #[test]
    fn convert_indexed_transparent_pixel() {
        // Arrange
        let palette = vec![255u8; 256 * 4];
        let indexed = [255u8]; // transparent index

        // Act
        let rgba = convert_indexed_to_rgba(&indexed, &palette, 255);

        // Assert
        assert_eq!(rgba[3], 0); // alpha = 0
    }

    #[test]
    fn convert_indexed_rgbx_palette_uses_native_channel_order() {
        // Arrange: palette entry 0 = R=0x40, G=0x80, B=0xFF, X=0
        let mut palette = vec![0u8; 4];
        palette[0] = 0x40; // R
        palette[1] = 0x80; // G
        palette[2] = 0xFF; // B
        let indexed = [0u8];

        // Act
        let rgba = convert_indexed_to_rgba(&indexed, &palette, 255);

        // Assert: RGBX channel order is retained
        assert_eq!(rgba[0], 0x40); // R
        assert_eq!(rgba[1], 0x80); // G
        assert_eq!(rgba[2], 0xFF); // B
    }

    #[test]
    fn convert_indexed_rgb_triples_use_native_channel_order() {
        // Arrange: PAL1-0.DAT stores 256 RGB triples.
        let mut palette = vec![0u8; 256 * 3];
        palette[7 * 3] = 0x22;
        palette[7 * 3 + 1] = 0x55;
        palette[7 * 3 + 2] = 0x88;

        // Act
        let rgba = convert_indexed_to_rgba(&[7], &palette, 255);

        // Assert
        assert_eq!(rgba, [0x22, 0x55, 0x88, 0xFF]);
    }

    // -- generate_minimap_rgba --

    #[test]
    fn generate_minimap_water_color() {
        // Arrange: all heights = 0 (water)
        let data = MinimapData {
            heights: [[0u16; 128]; 128],
            native_terrain_rgba: None,
            native_palette: None,
            dots: vec![],
        };

        // Act
        let rgba = generate_minimap_rgba(&data);

        // Assert: center uses the sampled native water and corners are transparent.
        let center = (64 * 128 + 64) * 4;
        assert_eq!(rgba[center], 0x00);
        assert_eq!(rgba[center + 1], 0x55);
        assert_eq!(rgba[center + 2], 0x6B);
        assert_eq!(rgba[center + 3], 255);
        assert_eq!(rgba[3], 0);
    }

    #[test]
    fn generate_minimap_land_gradient() {
        // Arrange: center cell height = 512
        let mut heights = [[0u16; 128]; 128];
        heights[64][64] = 512;
        let data = MinimapData {
            heights,
            native_terrain_rgba: None,
            native_palette: None,
            dots: vec![],
        };

        // Act
        let rgba = generate_minimap_rgba(&data);

        // Assert: green channel should be higher than water (40)
        let v = ((512.0f32 / 1024.0) * 180.0).min(255.0) as u8;
        let off = (64 * 128 + 64) * 4;
        assert_eq!(rgba[off], v / 4);
        assert_eq!(rgba[off + 1], 40 + v / 2);
        assert_eq!(rgba[off + 2], v / 6);
    }

    #[test]
    fn generate_minimap_person_marker_uses_native_fallback_colour_and_footprint() {
        // Arrange: water terrain, one centered red-tribe person marker.
        let data = MinimapData {
            heights: [[0u16; 128]; 128],
            native_terrain_rgba: None,
            native_palette: None,
            dots: vec![MinimapDot {
                cell_x: 64,
                cell_y: 64,
                tribe_index: 1,
                kind: MinimapMarkerKind::Person,
            }],
        };

        // Act
        let rgba = generate_minimap_rgba(&data);

        // Assert: the native person marker is a 2x2 #C7734B block.
        for (x, y) in [(64, 64), (65, 64), (64, 65), (65, 65)] {
            let off = (y * 128 + x) * 4;
            assert_eq!(&rgba[off..off + 4], &[0xC7, 0x73, 0x4B, 0xFF]);
        }
    }

    #[test]
    fn generate_minimap_building_marker_uses_its_distinct_native_palette_entry() {
        let mut palette = vec![0u8; 256 * 4];
        palette[0xDA * 4..0xDA * 4 + 3].copy_from_slice(&[0x23, 0x33, 0x6F]);
        let data = MinimapData {
            heights: [[0u16; 128]; 128],
            native_terrain_rgba: None,
            native_palette: Some(palette.into()),
            dots: vec![MinimapDot {
                cell_x: 64,
                cell_y: 64,
                tribe_index: 0,
                kind: MinimapMarkerKind::Building,
            }],
        };

        let rgba = generate_minimap_rgba(&data);
        for (x, y) in [(64, 64), (65, 64), (64, 65), (65, 65)] {
            let off = (y * 128 + x) * 4;
            assert_eq!(&rgba[off..off + 4], &[0x23, 0x33, 0x6F, 0xFF]);
        }
    }

    #[test]
    fn generate_minimap_wild_person_marker_is_a_single_native_palette_pixel() {
        let data = MinimapData {
            heights: [[0u16; 128]; 128],
            native_terrain_rgba: None,
            native_palette: None,
            dots: vec![MinimapDot {
                cell_x: 64,
                cell_y: 64,
                tribe_index: u8::MAX,
                kind: MinimapMarkerKind::WildPerson,
            }],
        };

        let rgba = generate_minimap_rgba(&data);
        let center = (64 * 128 + 64) * 4;
        let right = (64 * 128 + 65) * 4;
        assert_eq!(&rgba[center..center + 4], &[0x7F, 0x77, 0x57, 0xFF]);
        assert_eq!(&rgba[right..right + 4], &[0x00, 0x55, 0x6B, 0xFF]);
    }

    #[test]
    fn generate_minimap_prefers_native_terrain_when_available() {
        let mut terrain = vec![0u8; 128 * 128 * 4];
        let center = (64 * 128 + 64) * 4;
        terrain[center..center + 4].copy_from_slice(&[0x12, 0x34, 0x56, 0xff]);
        let data = MinimapData {
            heights: [[0u16; 128]; 128],
            native_terrain_rgba: Some(terrain.into()),
            native_palette: None,
            dots: vec![],
        };

        let rgba = generate_minimap_rgba(&data);

        assert_eq!(&rgba[center..center + 4], &[0x12, 0x34, 0x56, 0xff]);
    }

    // -- compute_hud_layout --

    #[test]
    fn compute_hud_layout_base_resolution() {
        // Arrange: 640x480 = 1x scale

        // Act
        let l = compute_hud_layout(640.0, 480.0);

        // Assert
        assert_eq!(l.sidebar_w, 100.0);
        assert_eq!(l.scale_x, 1.0);
        assert_eq!(l.scale_y, 1.0);
        assert_eq!(l.mm_pad, 0.0);
        assert_eq!(l.mm_size, 100.0);
        assert_eq!(l.mm_w, 100.0);
        assert_eq!(l.mm_h, 96.0);
        assert_eq!(l.tab_y, 81.0);
        assert_eq!(l.tab_h, 27.0);
        assert_eq!(l.tab_xs, [0.0, 31.0, 63.0]);
        assert_eq!(l.panel_y, 203.0);
        assert_eq!(l.construction_cell_w, 46.0);
        assert_eq!(l.construction_cell_h, 52.0);
    }

    #[test]
    fn compute_hud_layout_double_resolution() {
        // Arrange: 1280x960 = 2x scale

        // Act
        let l = compute_hud_layout(1280.0, 960.0);

        // Assert
        assert_eq!(l.sidebar_w, 200.0);
        assert_eq!(l.scale_x, 2.0);
        assert_eq!(l.scale_y, 2.0);
        assert_eq!(l.mm_pad, 0.0);
        assert_eq!(l.mm_size, 200.0);
        assert_eq!(l.mm_w, 200.0);
        assert_eq!(l.mm_h, 192.0);
        assert_eq!(l.tab_y, 163.0);
        assert_eq!(l.panel_y, 407.0);
    }

    #[test]
    fn compute_hud_layout_font_scale_minimum() {
        // Arrange: very small screen where the native font would be sub-pixel.
        let l = compute_hud_layout(320.0, 200.0);

        // Assert
        assert_eq!(l.font_scale, 10.0);
    }

    // -- detect_tab_click --

    #[test]
    fn detect_tab_click_buildings() {
        // Arrange
        let layout = compute_hud_layout(640.0, 480.0);
        // Click in the middle of the first tab
        let x = layout.tab_xs[0] + layout.tab_w * 0.5;
        let y = layout.tab_y + layout.tab_h * 0.5;

        // Act
        let result = detect_tab_click(x, y, &layout);

        // Assert
        assert_eq!(result, Some(HudTab::Buildings));
    }

    #[test]
    fn spell_and_follower_silhouettes_are_inert() {
        let layout = compute_hud_layout(640.0, 480.0);
        let y = layout.tab_y + layout.tab_h * 0.5;

        assert_eq!(
            detect_tab_click(layout.tab_xs[1] + layout.tab_w * 0.5, y, &layout),
            None
        );
        assert_eq!(
            detect_tab_click(layout.tab_xs[2] + layout.tab_w * 0.5, y, &layout),
            None
        );
    }

    #[test]
    fn detect_tab_click_outside_returns_none() {
        // Arrange
        let layout = compute_hud_layout(640.0, 480.0);

        // Act: click above tab bar
        let above = detect_tab_click(10.0, layout.tab_y - 5.0, &layout);
        // Click below tab bar
        let below = detect_tab_click(10.0, layout.tab_y + layout.tab_h + 5.0, &layout);
        // Click left of tabs
        let left = detect_tab_click(-1.0, layout.tab_y + 2.0, &layout);

        // Assert
        assert_eq!(above, None);
        assert_eq!(below, None);
        assert_eq!(left, None);
    }

    #[test]
    fn detect_first_construction_slot() {
        let layout = compute_hud_layout(640.0, 480.0);
        let cell = layout::element_rect(
            &layout::PANEL_TAB_PAGE,
            &layout::CONSTRUCTION_PAGE[0],
            layout.screen_w as i32,
            layout.screen_h as i32,
        );
        let result = detect_construction_slot_click(
            (cell.x + cell.w / 2) as f32,
            (cell.y + cell.h / 2) as f32,
            &layout,
        );
        assert_eq!(result, Some(0));
    }

    #[test]
    fn construction_slot_click_outside_grid_is_ignored() {
        let layout = compute_hud_layout(640.0, 480.0);
        assert_eq!(
            detect_construction_slot_click(layout.sidebar_w + 1.0, layout.panel_y + 1.0, &layout,),
            None
        );
    }

    #[test]
    fn construction_tab_uses_native_hfx_building_icons() {
        assert_eq!(
            HFX_CONSTRUCTION_ICONS,
            [1028, 1029, 1030, 1032, 1033, 1031, 1034, 1035, 1036]
        );
        assert_eq!(
            HFX_CONSTRUCTION_ICONS_HOVER,
            [1046, 1047, 1048, 1050, 1051, 1049, 1052, 1053, 1054]
        );
        assert_eq!(HFX_CONSTRUCTION_BLOCKED_OVERLAY, 1055);
        assert_eq!(construction_icon_sprite(0, false), Some(1028));
        assert_eq!(construction_icon_sprite(5, true), Some(1049));
        assert_eq!(construction_icon_sprite(9, false), None);
        assert_eq!(FONT4_HUD_GLYPH_IDS, [FONT4_STATUS_GLYPH_I]);
        assert_eq!(
            HFX_BUILDING_FRAME,
            [821, 825, 822, 827, 829, 828, 823, 826, 824]
        );
        assert_eq!(
            HFX_BUILDING_FRAME_HOVER,
            [839, 843, 840, 845, 847, 846, 841, 844, 842]
        );
        assert_eq!(
            HFX_BUILDING_FRAME_PRESSED,
            [830, 834, 831, 836, 838, 837, 832, 835, 833]
        );
        assert_eq!(
            HFX_PANEL_SURFACE_TILES,
            [
                1450, 1451, 1452, 1453, 1454, 1455, 1456, 1457, 1458, 1459, 1460, 1461, 1462, 1463,
                1464, 1465,
            ]
        );
    }

    #[test]
    fn construction_command_ids_follow_native_icons_not_panel_commands() {
        assert_eq!(
            (0..layout::CONSTRUCTION_PAGE.len())
                .map(construction_command_for_slot)
                .collect::<Vec<_>>(),
            vec![
                Some(1),
                Some(4),
                Some(7),
                Some(5),
                Some(6),
                Some(8),
                Some(13),
                Some(15),
                Some(17),
            ]
        );
        // Panel control ids intentionally differ, e.g. the first cell is
        // cmd 2 but represents construction command 1 (Small Hut).
        assert_eq!(layout::CONSTRUCTION_PAGE[0].cmd, 2);
    }

    #[test]
    fn level_building_subtypes_match_house_tab_commands() {
        assert_eq!(construction_command_for_level_building_subtype(1), Some(1));
        assert_eq!(construction_command_for_level_building_subtype(3), Some(1));
        assert_eq!(construction_command_for_level_building_subtype(4), Some(4));
        assert_eq!(construction_command_for_level_building_subtype(7), Some(7));
        assert_eq!(
            construction_command_for_level_building_subtype(13),
            Some(13)
        );
        assert_eq!(
            construction_command_for_level_building_subtype(15),
            Some(15)
        );
        assert_eq!(
            construction_command_for_level_building_subtype(18),
            Some(17)
        );
        assert_eq!(construction_command_for_level_building_subtype(14), None);
    }

    #[test]
    fn construction_availability_matches_native_level_one_pattern() {
        let available = construction_command_bit(1) | construction_command_bit(17);
        let present = available | construction_command_bit(7);

        assert_eq!(
            construction_slot_availability(0, available, present),
            ConstructionSlotAvailability::Available
        );
        assert_eq!(
            construction_slot_availability(2, available, present),
            ConstructionSlotAvailability::Blocked
        );
        assert_eq!(
            construction_slot_availability(8, available, present),
            ConstructionSlotAvailability::Available
        );
        assert_eq!(
            construction_slot_availability(1, available, present),
            ConstructionSlotAvailability::Hidden
        );
    }

    #[test]
    fn tiled_panel_interior_cycle_matches_native_border_offset() {
        // GUI_RenderTiledPanel starts both counters at the top-left border.
        // Its first interior cell is therefore source tile 1465, then the
        // cycle proceeds 1463 / 1464 / 1462.  This protects the native
        // mottled-panel texture from a visibly shifted checkerboard seam.
        let cycle = [
            panel_surface_interior_tile(&HFX_PANEL_SURFACE_TILES, 0, 0),
            panel_surface_interior_tile(&HFX_PANEL_SURFACE_TILES, 0, 1),
            panel_surface_interior_tile(&HFX_PANEL_SURFACE_TILES, 1, 0),
            panel_surface_interior_tile(&HFX_PANEL_SURFACE_TILES, 1, 1),
        ];
        assert_eq!(cycle, [1465, 1463, 1464, 1462]);
    }

    #[test]
    fn construction_tab_hfx_assets_include_both_frame_states_and_all_icons() {
        assert_eq!(HFX_TAB_ICONS, [676, 678, 680]);
        assert_eq!(HFX_HUD_SPRITE_IDS.len(), 137);

        for sprite_id in HFX_TAB_FRAME
            .iter()
            .chain(HFX_TAB_FRAME_SELECTED.iter())
            .chain(HFX_BUILDING_FRAME.iter())
            .chain(HFX_BUILDING_FRAME_HOVER.iter())
            .chain(HFX_BUILDING_FRAME_PRESSED.iter())
            .chain(HFX_CONSTRUCTION_ICONS.iter())
            .chain(HFX_CONSTRUCTION_ICONS_HOVER.iter())
            .chain(std::iter::once(&HFX_CONSTRUCTION_BLOCKED_OVERLAY))
            .chain(HFX_STATUS_AVATAR_FRAME.iter())
            .chain(HFX_STATUS_GLOBE_FRAME.iter())
            .chain(HFX_STATUS_SMALL_FRAME.iter())
            .chain(HFX_STATUS_TALL_FRAME.iter())
            .chain(HFX_TAB_ICONS.iter())
            .chain(std::iter::once(&HFX_TAB_ICON_BUILDINGS_SELECTED))
            .chain(
                [
                    HFX_STATUS_AVATAR_COMPOSITE,
                    HFX_STATUS_BLACK_TEXTURE,
                    HFX_STATUS_WHITE_TEXTURE,
                    HFX_STATUS_GLOBE,
                    HFX_STATUS_HELP_GLYPH,
                    HFX_STATUS_BLUE_CHIP,
                    HFX_STATUS_RED_CHIP,
                    HFX_STATUS_FOLLOWER_GLYPH,
                    HFX_POPULATION_METER,
                ]
                .iter(),
            )
            .chain(HFX_PANEL_SURFACE_TILES.iter())
        {
            assert!(
                HFX_HUD_SPRITE_IDS.contains(sprite_id),
                "HFX sprite {sprite_id} must be packed into the HUD atlas"
            );
        }

        assert_eq!(HSPR_HUD_SPRITE_IDS, [HSPR_STATUS_AVATAR_BLUE]);

        for sprite_id in HFX_MINIMAP_FRAME
            .into_iter()
            .filter(|&sprite_id| sprite_id != 0)
            .chain(std::iter::once(HFX_SHAMAN_WIDGET))
        {
            assert!(
                HFX_HUD_SPRITE_IDS.contains(&sprite_id),
                "HFX sprite {sprite_id} must be packed into the HUD atlas"
            );
        }
    }

    #[test]
    fn construction_button_frames_match_original_renderer_state_tables() {
        assert_eq!(
            construction_button_frame(ConstructionButtonState::Normal),
            &HFX_BUILDING_FRAME
        );
        assert_eq!(
            construction_button_frame(ConstructionButtonState::Hovered),
            &HFX_BUILDING_FRAME_HOVER
        );
        assert_eq!(
            construction_button_frame(ConstructionButtonState::Pressed),
            &HFX_BUILDING_FRAME_PRESSED
        );
    }

    #[test]
    fn construction_button_state_requires_same_slot_for_pressed_art() {
        assert_eq!(
            construction_button_state(3, None, None),
            ConstructionButtonState::Normal
        );
        assert_eq!(
            construction_button_state(3, Some(3), None),
            ConstructionButtonState::Hovered
        );
        assert_eq!(
            construction_button_state(3, Some(3), Some(3)),
            ConstructionButtonState::Pressed
        );
        assert_eq!(
            construction_button_state(3, Some(4), Some(3)),
            ConstructionButtonState::Normal
        );
    }

    // -- panel_sprite_index --

    #[test]
    fn panel_sprite_index_calculation() {
        // Arrange
        let font_region_start = 1;
        let psfb_index = 5;

        // Act
        let idx = panel_sprite_index(font_region_start, psfb_index);

        // Assert: 1 + 96 + 5 = 102
        assert_eq!(idx, 102);
    }

    // -- compute_mana_fraction --

    #[test]
    fn mana_fraction_zero_max_returns_zero() {
        assert_eq!(compute_mana_fraction(500, 0), 0.0);
    }

    #[test]
    fn mana_fraction_full_returns_one() {
        assert_eq!(compute_mana_fraction(1_000_000, 1_000_000), 1.0);
    }

    #[test]
    fn mana_fraction_half() {
        assert_eq!(compute_mana_fraction(500_000, 1_000_000), 0.5);
    }

    #[test]
    fn mana_fraction_overflow_clamped() {
        assert_eq!(compute_mana_fraction(2_000_000, 1_000_000), 1.0);
    }

    // -- SpellCooldown --

    #[test]
    fn spell_cooldown_fraction() {
        let cd = SpellCooldown {
            spell_index: 0,
            cooldown_remaining: 100,
            cooldown_total: 200,
        };
        assert_eq!(cd.cooldown_remaining as f32 / cd.cooldown_total as f32, 0.5);
    }

    // -- minimap_click_to_cell --

    #[test]
    fn click_to_cell_center() {
        let (cx, cy) = minimap_click_to_cell(64.0, 48.0, 0.0, 0.0, 128.0, 96.0);
        assert_eq!(cx, 64.0);
        assert_eq!(cy, 64.0);
    }

    #[test]
    fn click_to_cell_origin() {
        let (cx, cy) = minimap_click_to_cell(10.0, 10.0, 10.0, 10.0, 128.0, 96.0);
        assert_eq!(cx, 0.0);
        assert_eq!(cy, 0.0);
    }

    #[test]
    fn click_to_cell_clamped() {
        let (cx, _) = minimap_click_to_cell(0.0, 0.0, 10.0, 10.0, 128.0, 96.0);
        assert_eq!(cx, 0.0); // clamped, not negative
    }

    // -- toroidal_delta --

    #[test]
    fn toroidal_delta_short_positive() {
        assert_eq!(toroidal_delta(10.0, 30.0), 20.0);
    }

    #[test]
    fn toroidal_delta_short_negative() {
        assert_eq!(toroidal_delta(30.0, 10.0), -20.0);
    }

    #[test]
    fn toroidal_delta_wraps_positive() {
        // from 120 to 5: shortest path is +13 (120->128->5), not -115
        let d = toroidal_delta(120.0, 5.0);
        assert!((d - 13.0).abs() < 0.01);
    }

    #[test]
    fn toroidal_delta_wraps_negative() {
        // from 5 to 120: shortest path is -13 (5->0->120), not +115
        let d = toroidal_delta(5.0, 120.0);
        assert!((d - (-13.0)).abs() < 0.01);
    }

    // -- unit_subtype_name --

    #[test]
    fn subtype_brave_name() {
        assert_eq!(unit_subtype_name(2), "Brave");
    }

    #[test]
    fn subtype_warrior_name() {
        assert_eq!(unit_subtype_name(3), "Warrior");
    }

    #[test]
    fn subtype_shaman_name() {
        assert_eq!(unit_subtype_name(7), "Shaman");
    }

    #[test]
    fn subtype_unknown_fallback() {
        assert_eq!(unit_subtype_name(255), "Unknown");
    }

    #[test]
    fn health_bar_entry_fraction() {
        let hb = HealthBarEntry {
            screen_x: 100.0,
            screen_y: 50.0,
            health_fraction: 0.5,
            bar_type: HealthBarType::Unit,
        };
        assert_eq!(hb.health_fraction, 0.5);
    }

    #[test]
    fn health_bar_type_copy() {
        let t = HealthBarType::Unit;
        let t2 = t; // Copy trait
        assert!(matches!(t2, HealthBarType::Unit));
    }

    #[test]
    fn health_bar_type_building() {
        let t = HealthBarType::Building;
        let t2 = t;
        assert!(matches!(t2, HealthBarType::Building));
    }
}
