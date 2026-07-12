use cgmath::Vector4;

use crate::data::objects::{Object3D, Shape};
use crate::engine::units::{DragState, UnitCoordinator};
use crate::render::camera::{Camera, Screen};
use crate::render::hud::HudState;
use crate::render::sprites::LevelObject;
use crate::render::terrain::LandscapeMesh;

/// Ghost preview state for building placement UI.
/// Used by the renderer to draw a transparent building mesh at the mouse position.
#[derive(Debug, Clone, Copy)]
pub struct GhostPreviewState {
    pub building_type: u8,
    pub cell_x: i32,
    pub cell_y: i32,
    pub rotation: u8,
    pub valid: bool,
}

/// Output boundary — everything the renderer needs to produce one frame.
/// Produced by GameEngine, consumed by Renderer.
pub struct FrameState<'a> {
    // View
    pub camera: &'a Camera,
    pub screen: &'a Screen,
    pub zoom: f32,

    // Landscape
    pub landscape: &'a LandscapeMesh<128>,
    pub curvature_scale: f32, // 0.0 if disabled
    pub sunlight: Vector4<f32>,
    pub wat_offset: i32,

    // Objects
    pub show_objects: bool,
    pub show_shadows: bool,
    pub show_lighting: bool,
    pub show_markers: bool,
    pub unit_coordinator: &'a UnitCoordinator,
    pub level_objects: &'a [LevelObject],
    pub building_objects: &'a [Option<Object3D>],
    pub scenery_objects: &'a [Option<Object3D>],
    pub shapes: &'a [Shape],

    // Buildings (new)
    pub ghost_preview: Option<GhostPreviewState>,
    pub needs_building_rebuild: bool,

    // HUD
    pub hud_state: HudState,
    pub drag_state: &'a DragState,

    // Dirty flags (set by apply_command, cleared after renderer processes them)
    pub needs_spawn_rebuild: bool,
    pub needs_unit_rebuild: bool,
    pub needs_level_reload: bool,
}
