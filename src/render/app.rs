use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes};

use cgmath::{Point2, Point3, Vector3, Vector4, Matrix4, SquareMatrix};

use crate::render::tex_model::TexModel;
use crate::render::model::{MeshModel, VertexModel};
use crate::render::color_model::{ColorModel, ColorVertex};
use crate::render::camera::*;

use crate::data::psfb::ContainerPSFB;
use crate::data::types::BinDeserializer;
use crate::data::animation::{
    AnimationsData, AnimationSequence,
    build_multi_anim_atlas, build_direct_multi_anim_atlas,
    UNIT_MULTI_ANIMS, SHAMAN_ANIMS,
};
use crate::engine::state::constants::*;
use crate::engine::movement::constants::CELL_HAS_BUILDING;

use crate::render::picking::intersect_iter;

use crate::render::terrain::{
    LandscapeMesh, LandscapeModel,
    LandscapeUniformData, LandscapeVariant, LandscapeProgramContainer,
    make_landscape_model, LANDSCAPE_SCALE, LANDSCAPE_OFFSET,
};
use crate::data::level::{LevelRes, ObjectPaths};
use crate::data::units::{ModelType, object_3d_index};
use crate::data::objects::{Object3D, Shape, ShapeFootprints};
use crate::data::bl320::make_bl320_texture_rgba;
use crate::data::landscape::{make_texture_land, draw_texture_u8};

use crate::engine::units::{UnitCoordinator, DragState, Unit};
use crate::engine::units::coords::{cell_to_world, cell_to_tile, triangle_to_cell, project_to_screen, ScreenRect};
use crate::render::buildings::build_building_meshes;
use crate::render::sprites::{
    LevelObject, UnitTypeRender,
    obj_colors, convert_palette,
    pack_palette_rgba, rgb_to_rgba,
    extract_level_objects,
    build_spawn_model,
    build_object_markers, build_unit_markers, build_selection_outlines,
};

use crate::render::gpu::context::GpuContext;
use crate::render::gpu::pipeline::{create_pipeline, create_pipeline_blended};
use crate::render::gpu::buffer::GpuBuffer;
use crate::render::gpu::texture::GpuTexture;
use crate::render::gpu::bind_groups::{
    create_landscape_group0_layout, create_objects_group0_layout,
    create_objects_group1_layout, make_storage_entry,
};
use crate::render::envelop::*;

use crate::engine::state::tick::{GameWorld, StdTimeSource, TickSubsystems};
use crate::engine::state::state_machine::GameState;
use crate::engine::state::traits::NoOp;
use crate::engine::state::mana_tick::ManaTickBridge;
use crate::engine::{GameCommand, FrameState, translate_key};

use crate::render::hud::{
    self, HudTab, HudState, HudRenderer,
    MinimapData, MinimapDot, PanelEntry, TribePopulation,
    HUD_TRIBE_COLORS,
};

/******************************************************************************/

type LandscapeMeshS = LandscapeMesh<128>;


/******************************************************************************/
// Overlay text rendering — minimal bitmap font


fn help_text() -> &'static str {
    concat!(
        "Q/E:    Rotate\n",
        "Up/Dn:  Tilt\n",
        "WASD:   Pan terrain\n",
        "Space:  Center on blue\n",
        "B/V:    Next/Prev level\n",
        "N/M:    Next/Prev shader\n",
        "C:      Toggle curvature\n",
        "[/]:    Curvature +/-\n",
        "O:      Toggle objects\n",
        "G:      Toggle shadows+lighting\n",
        "U:      Toggle unit markers\n",
        "F1:     Toggle HUD\n",
        "Scroll: Zoom\n",
        "Esc:    Quit",
    )
}

/******************************************************************************/

#[rustfmt::skip]
const OPENGL_TO_WGPU: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

fn compute_light_mvp(sunlight: &Vector4<f32>, focus: Point3<f32>, radius: f32) -> Matrix4<f32> {
    let lx = sunlight.x;
    let ly = sunlight.y;
    let len = (lx * lx + ly * ly + 200.0 * 200.0_f32).sqrt();
    let dir = Vector3::new(-lx / len, -ly / len, 200.0 / len);

    let eye = Point3::new(
        focus.x + dir.x * radius,
        focus.y + dir.y * radius,
        focus.z + dir.z * radius,
    );
    let up = if dir.z.abs() > 0.99 {
        Vector3::unit_y()
    } else {
        Vector3::unit_z()
    };
    let light_view = Matrix4::look_at_rh(eye, focus, up);
    let half = radius * 0.6;
    let light_proj = cgmath::ortho(-half, half, -half, half, 0.1, radius * 2.5);
    OPENGL_TO_WGPU * light_proj * light_view
}

/******************************************************************************/

pub struct AppConfig {
    pub base: Option<PathBuf>,
    pub level: Option<u8>,
    pub landtype: Option<String>,
    pub cpu: bool,
    pub cpu_full: bool,
    pub debug: bool,
    pub light: Option<(i16, i16)>,
    pub script: Option<PathBuf>,
}

/// All game-logic state — no GPU types. Produces FrameState for the renderer.
pub struct GameEngine {
    landscape_mesh: LandscapeMeshS,
    camera: Camera,
    screen: Screen,
    curvature_scale: f32,
    curvature_enabled: bool,
    zoom: f32,
    level_num: u8,
    sunlight: Vector4<f32>,
    show_objects: bool,
    show_shadows: bool,
    show_lighting: bool,
    show_markers: bool,
    sprite_z_offset: f32,
    sprite_scale: f32,
    hud_tab: HudTab,
    hud_visible: bool,
    compass_visible: bool,
    walkability_visible: bool,
    hud_panel_sprite_count: usize,

    // Game simulation
    unit_coordinator: UnitCoordinator,
    game_world: GameWorld,
    game_time: StdTimeSource,

    // Level data
    level_objects: Vec<LevelObject>,
    building_objects: Vec<Option<Object3D>>,  // from OBJS bank 0 (building models)
    scenery_objects: Vec<Option<Object3D>>,   // from level-specific OBJS bank (scenery models)
    shapes: Vec<Shape>,
    shape_footprints: ShapeFootprints,

    // Water animation
    wat_offset: i32,
    wat_interval: u32,
    frame_count: u32,

    // Config (read-only after init)
    config: AppConfig,
}

/// Raw input state — mouse position and drag tracking.
/// Lives on App, not GameEngine, because it's I/O-layer state.
pub struct InputState {
    mouse_pos: Point2<f32>,
    drag_state: DragState,
}

impl GameEngine {
    fn reset_camera(&mut self) {
        self.camera.angle_x = -55;
        self.camera.angle_y = 0;
        self.camera.angle_z = 0;
        self.zoom = 1.0;
    }

    fn build_landscape_params(&self) -> LandscapeUniformData {
        let shift = self.landscape_mesh.get_shift_vector();
        LandscapeUniformData {
            level_shift: [shift.x, shift.y, shift.z, shift.w],
            height_scale: self.landscape_mesh.height_scale(),
            step: self.landscape_mesh.step(),
            width: self.landscape_mesh.width() as i32,
            _pad_width: 0,
            sunlight: [self.sunlight.x, self.sunlight.y, self.sunlight.z, self.sunlight.w],
            wat_offset: self.wat_offset,
            curvature_scale: if self.curvature_enabled { self.curvature_scale } else { 0.0 },
            camera_focus: {
                let center = (self.landscape_mesh.width() - 1) as f32 * self.landscape_mesh.step() / 2.0;
                [center, center]
            },
            viewport_radius: {
                let center = (self.landscape_mesh.width() - 1) as f32 * self.landscape_mesh.step() / 2.0;
                center * 0.9
            },
            _pad2: [0.0; 3],
        }
    }

    /// World-space center of the terrain (accounting for model transform).
    fn world_center(&self) -> f32 {
        let center_model = (self.landscape_mesh.width() - 1) as f32 * self.landscape_mesh.step() / 2.0;
        LANDSCAPE_SCALE * center_model + LANDSCAPE_OFFSET
    }

    fn camera_focus_vertex(&self) -> f32 {
        let center_model = (self.landscape_mesh.width() - 1) as f32 * self.landscape_mesh.step() / 2.0;
        center_model / self.landscape_mesh.step()
    }

    fn camera_min_z(&self) -> f32 {
        let center = self.world_center();
        let az = (self.camera.angle_z as f32).to_radians();
        let ax = (self.camera.angle_x as f32).to_radians();
        let radius = 1.5 / self.zoom;
        let eye_x = center + radius * ax.cos() * az.sin();
        let eye_y = center + radius * ax.cos() * az.cos();
        // Convert world-space eye position back to grid coords for height lookup
        let model_x = (eye_x - LANDSCAPE_OFFSET) / LANDSCAPE_SCALE;
        let model_y = (eye_y - LANDSCAPE_OFFSET) / LANDSCAPE_SCALE;
        let step = self.landscape_mesh.step();
        let n = self.landscape_mesh.width();
        let gx = (model_x / step).clamp(0.0, (n - 1) as f32) as usize;
        let gy = (model_y / step).clamp(0.0, (n - 1) as f32) as usize;
        let shift = self.landscape_mesh.get_shift_vector();
        let sx = (gx + shift.x as usize) % n;
        let sy = (gy + shift.y as usize) % n;
        self.landscape_mesh.height_at(sx, sy) as f32 * self.landscape_mesh.height_scale() + 0.05
    }

    fn screen_to_cell(&self, mouse_pos: &Point2<f32>) -> Option<(f32, f32)> {
        let center = self.world_center();
        let focus = Vector3::new(center, center, 0.0);
        let min_z = self.camera_min_z();
        let (v1, v2) = screen_to_scene_zoom(&self.screen, &self.camera, mouse_pos, self.zoom, focus, min_z);
        let mvp_transform = Matrix4::from_translation(Vector3::new(LANDSCAPE_OFFSET, LANDSCAPE_OFFSET, 0.0))
            * Matrix4::from_scale(LANDSCAPE_SCALE);
        let iter = self.landscape_mesh.iter();
        match intersect_iter(iter, &mvp_transform, v1, v2) {
            Some((triangle_id, _)) => {
                let shift = self.landscape_mesh.get_shift_vector();
                Some(triangle_to_cell(
                    triangle_id,
                    self.landscape_mesh.width(),
                    shift.x as usize,
                    shift.y as usize,
                ))
            }
            None => None,
        }
    }

    fn unit_pvm(&self) -> Matrix4<f32> {
        let center = self.world_center();
        let focus = Vector3::new(center, center, 0.0);
        let min_z = self.camera_min_z();
        let mvp = MVP::with_zoom(&self.screen, &self.camera, self.zoom, focus, min_z);
        let model_transform = Matrix4::from_translation(Vector3::new(LANDSCAPE_OFFSET, LANDSCAPE_OFFSET, 0.0))
            * Matrix4::from_scale(LANDSCAPE_SCALE);
        mvp.projection * mvp.view * model_transform
    }

    fn unit_screen_pos(&self, unit: &Unit, pvm: &Matrix4<f32>) -> Option<(f32, f32)> {
        let step = self.landscape_mesh.step();
        let w = self.landscape_mesh.width() as f32;
        let shift = self.landscape_mesh.get_shift_vector();
        let height_scale = self.landscape_mesh.height_scale();
        let center = (w - 1.0) * step / 2.0;
        let cs = if self.curvature_enabled { self.curvature_scale } else { 0.0 };
        let vis_x = ((unit.cell_x - shift.x as f32) % w + w) % w;
        let vis_y = ((unit.cell_y - shift.y as f32) % w + w) % w;
        let gx = vis_x * step;
        let gy = vis_y * step;
        let ix = (unit.cell_x as usize).min(127);
        let iy = (unit.cell_y as usize).min(127);
        let gz = self.landscape_mesh.height_at(ix, iy) as f32 * height_scale;
        let dx = gx - center;
        let dy = gy - center;
        let curvature_offset = (dx * dx + dy * dy) * cs;
        let z_base = gz - curvature_offset;
        project_to_screen([gx, gy, z_base], pvm, self.screen.width as f32, self.screen.height as f32)
    }

    /// Compute the billboard's screen-space AABB for a unit.
    /// Uses the same billboard geometry as `build_unit_markers`.
    fn unit_screen_rect(&self, unit: &Unit, pvm: &Matrix4<f32>, right: &Vector3<f32>, up: &Vector3<f32>) -> Option<ScreenRect> {
        let step = self.landscape_mesh.step();
        let w = self.landscape_mesh.width() as f32;
        let shift = self.landscape_mesh.get_shift_vector();
        let height_scale = self.landscape_mesh.height_scale();
        let center = (w - 1.0) * step / 2.0;
        let cs = if self.curvature_enabled { self.curvature_scale } else { 0.0 };
        let vis_x = ((unit.cell_x - shift.x as f32) % w + w) % w;
        let vis_y = ((unit.cell_y - shift.y as f32) % w + w) % w;
        let gx = vis_x * step;
        let gy = vis_y * step;
        let ix = (unit.cell_x as usize).min(127);
        let iy = (unit.cell_y as usize).min(127);
        let gz = self.landscape_mesh.height_at(ix, iy) as f32 * height_scale;
        let dx = gx - center;
        let dy = gy - center;
        let curvature_offset = (dx * dx + dy * dy) * cs;
        let z_base = gz - curvature_offset;

        let half_w = step * 0.15;
        let sprite_h = step * 0.4;
        let base = Vector3::new(gx, gy, z_base);
        let bl = base - right * half_w;
        let br = base + right * half_w;
        let tl = bl + up * sprite_h;
        let tr = br + up * sprite_h;

        let sw = self.screen.width as f32;
        let sh = self.screen.height as f32;
        let s_bl = project_to_screen([bl.x, bl.y, bl.z], pvm, sw, sh)?;
        let s_br = project_to_screen([br.x, br.y, br.z], pvm, sw, sh)?;
        let s_tl = project_to_screen([tl.x, tl.y, tl.z], pvm, sw, sh)?;
        let s_tr = project_to_screen([tr.x, tr.y, tr.z], pvm, sw, sh)?;

        let min_x = s_bl.0.min(s_br.0).min(s_tl.0).min(s_tr.0);
        let max_x = s_bl.0.max(s_br.0).max(s_tl.0).max(s_tr.0);
        let min_y = s_bl.1.min(s_br.1).min(s_tl.1).min(s_tr.1);
        let max_y = s_bl.1.max(s_br.1).max(s_tl.1).max(s_tr.1);

        Some(ScreenRect { min_x, min_y, max_x, max_y })
    }

    /// Compute the view matrix right/up vectors for billboard orientation.
    fn billboard_axes(&self) -> (Vector3<f32>, Vector3<f32>) {
        let center = self.world_center();
        let az = (self.camera.angle_z as f32).to_radians();
        let ax = (self.camera.angle_x as f32).to_radians();
        let eye = Point3::new(
            center + ax.cos() * az.sin(),
            center + ax.cos() * az.cos(),
            -ax.sin(),
        );
        let target = Point3::new(center, center, 0.0);
        let view = Matrix4::look_at_rh(eye, target, Vector3::new(0.0, 0.0, 1.0));
        let right = Vector3::new(view.x.x, view.y.x, view.z.x);
        let up = Vector3::new(view.x.y, view.y.y, view.z.y);
        (right, up)
    }

    fn find_unit_at_screen_pos(&self, mouse: &Point2<f32>) -> Option<usize> {
        let pvm = self.unit_pvm();
        let (right, up) = self.billboard_axes();
        let mut best: Option<(usize, f32)> = None;
        for unit in self.unit_coordinator.units() {
            if let Some(rect) = self.unit_screen_rect(unit, &pvm, &right, &up) {
                if rect.contains(mouse.x, mouse.y) {
                    let (cx, cy) = rect.center();
                    let dist_sq = (cx - mouse.x).powi(2) + (cy - mouse.y).powi(2);
                    if best.is_none() || dist_sq < best.unwrap().1 {
                        best = Some((unit.id, dist_sq));
                    }
                }
            }
        }
        best.map(|(id, _)| id)
    }

    fn units_in_screen_rect(&self, corner_a: Point2<f32>, corner_b: Point2<f32>) -> Vec<usize> {
        let drag_rect = ScreenRect {
            min_x: corner_a.x.min(corner_b.x),
            max_x: corner_a.x.max(corner_b.x),
            min_y: corner_a.y.min(corner_b.y),
            max_y: corner_a.y.max(corner_b.y),
        };
        let pvm = self.unit_pvm();
        let (right, up) = self.billboard_axes();
        let mut ids = Vec::new();
        for unit in self.unit_coordinator.units() {
            if let Some(rect) = self.unit_screen_rect(unit, &pvm, &right, &up) {
                if rect.overlaps(&drag_rect) {
                    ids.push(unit.id);
                }
            }
        }
        ids
    }

    fn build_hud_state(&self) -> HudState {
        let dots: Vec<MinimapDot> = self.unit_coordinator.units().iter()
            .filter(|u| u.alive)
            .map(|u| MinimapDot {
                cell_x: (u.cell_x as u8).min(127),
                cell_y: (u.cell_y as u8).min(127),
                tribe_index: u.tribe_index,
            })
            .collect();
        let minimap = MinimapData {
            heights: *self.landscape_mesh.heights(),
            dots,
        };
        let panel_entries = match self.hud_tab {
            HudTab::Spells => {
                ["Burn", "Blast", "Lightning", "Whirlwind",
                 "Plague", "Invisibility", "Firestorm", "Hypnotize",
                 "Ghost Army", "Erosion", "Swamp", "Land Bridge",
                 "Angel/Death", "Earthquake", "Flatten", "Volcano"]
                    .iter()
                    .map(|name| PanelEntry {
                        label: name.to_string(),
                        color: [0.8, 0.9, 1.0, 0.9],
                    })
                    .collect()
            }
            HudTab::Buildings => {
                ["Hut", "Guard Tower", "Temple", "Spy Hut",
                 "Warrior Hut", "Firewarrior Hut", "Prison", "Boat Hut"]
                    .iter()
                    .map(|name| PanelEntry {
                        label: name.to_string(),
                        color: [0.9, 0.85, 0.7, 0.9],
                    })
                    .collect()
            }
            HudTab::Units => {
                let unit_types: [(u8, &str); 6] = [
                    (PERSON_SUBTYPE_BRAVE, "Brave"),
                    (PERSON_SUBTYPE_WARRIOR, "Warrior"),
                    (PERSON_SUBTYPE_SPY, "Spy"),
                    (PERSON_SUBTYPE_PREACHER, "Preacher"),
                    (PERSON_SUBTYPE_FIREWARRIOR, "Firewarr"),
                    (PERSON_SUBTYPE_SHAMAN, "Shaman"),
                ];
                unit_types.iter().map(|(subtype, name)| {
                    let count = self.unit_coordinator.units().iter()
                        .filter(|u| u.alive && u.subtype == *subtype && u.tribe_index == 0)
                        .count();
                    PanelEntry {
                        label: format!("{}: {}", name, count),
                        color: [0.7, 1.0, 0.7, 0.9],
                    }
                }).collect()
            }
        };
        let mut tribe_counts = [0u32; 4];
        for u in self.unit_coordinator.units() {
            if u.alive && (u.tribe_index as usize) < 4 {
                tribe_counts[u.tribe_index as usize] += 1;
            }
        }
        let tribe_populations: Vec<TribePopulation> = (0..4u8)
            .filter(|&t| tribe_counts[t as usize] > 0)
            .map(|t| TribePopulation {
                tribe_index: t,
                count: tribe_counts[t as usize],
                color: HUD_TRIBE_COLORS[t as usize],
            })
            .collect();
        HudState {
            active_tab: self.hud_tab,
            minimap,
            panel_entries,
            tribe_populations,
            level_num: self.level_num as u32,
            frame_count: self.frame_count as u64,
        }
    }

    /// Process a game command. Returns true if the renderer needs to redraw.
    /// Sets dirty flags for specific rebuilds.
    fn apply_command(&mut self, cmd: &GameCommand) -> bool {
        match cmd {
            GameCommand::RotateCamera { delta_z } => {
                self.camera.angle_z += delta_z;
                true
            }
            GameCommand::TiltCamera { delta_x } => {
                self.camera.angle_x = (self.camera.angle_x + delta_x).clamp(-90, -30);
                true
            }
            GameCommand::PanScreen { forward, right } => {
                let az = (self.camera.angle_z as f32).to_radians();
                let gx = -right * az.cos() - forward * az.sin();
                let gy = right * az.sin() - forward * az.cos();
                self.landscape_mesh.shift_x(gx.round() as i32);
                self.landscape_mesh.shift_y(gy.round() as i32);
                true
            }
            GameCommand::PanTerrain { dx, dy } => {
                self.landscape_mesh.shift_x(*dx);
                self.landscape_mesh.shift_y(*dy);
                true
            }
            GameCommand::ResetCamera => {
                self.reset_camera();
                true
            }
            GameCommand::TopDownView => {
                self.camera.angle_x = -90;
                true
            }
            GameCommand::CenterOnShaman => {
                // Needs unit_renders (App-level data). The actual centering
                // is done by App after apply_command returns.
                true
            }
            GameCommand::SetZoom(z) => {
                self.zoom = z.clamp(0.3, 5.0);
                true
            }
            GameCommand::ToggleCurvature => {
                self.curvature_enabled = !self.curvature_enabled;
                log::info!("curvature {}", if self.curvature_enabled { "on" } else { "off" });
                true
            }
            GameCommand::AdjustCurvature { factor } => {
                self.curvature_scale *= factor;
                log::info!("curvature_scale = {:.6}", self.curvature_scale);
                true
            }
            GameCommand::AdjustSpriteOffset { delta } => {
                self.sprite_z_offset += delta;
                eprintln!("[SPRITE] z_offset={:.4} scale={:.2}", self.sprite_z_offset, self.sprite_scale);
                true
            }
            GameCommand::AdjustSpriteScale { delta } => {
                self.sprite_scale = (self.sprite_scale + delta).max(0.05);
                eprintln!("[SPRITE] z_offset={:.4} scale={:.2}", self.sprite_z_offset, self.sprite_scale);
                true
            }
            GameCommand::NextLevel => {
                self.level_num = (self.level_num + 1) % 26;
                if self.level_num == 0 { self.level_num = 1; }
                true
            }
            GameCommand::PrevLevel => {
                self.level_num = if self.level_num == 1 { 25 } else { self.level_num - 1 };
                true
            }
            GameCommand::NextShader | GameCommand::PrevShader => {
                // Shader cycling stays renderer-side (program_container is GPU state)
                true
            }
            GameCommand::ToggleObjects => {
                self.show_objects = !self.show_objects;
                log::info!("objects {}", if self.show_objects { "on" } else { "off" });
                true
            }
            GameCommand::ToggleShadows => {
                self.show_shadows = !self.show_shadows;
                self.show_lighting = !self.show_lighting;
                log::info!("shadows+lighting {}", if self.show_shadows { "on" } else { "off" });
                true
            }
            GameCommand::ToggleMarkers => {
                self.show_markers = !self.show_markers;
                log::info!("markers {}", if self.show_markers { "on" } else { "off" });
                true
            }
            GameCommand::AdjustSunlight { dx, dy } => {
                self.sunlight.x += dx;
                self.sunlight.y += dy;
                log::debug!("sunlight = {:?}", self.sunlight);
                true
            }
            GameCommand::SelectUnit(id) => {
                self.unit_coordinator.selection.select_single(*id);
                true
            }
            GameCommand::SelectMultiple(ids) => {
                self.unit_coordinator.selection.select_multiple(ids.clone());
                true
            }
            GameCommand::ClearSelection => {
                self.unit_coordinator.selection.clear();
                true
            }
            GameCommand::OrderMove { x, z } => {
                let target = crate::engine::movement::WorldCoord::new(*x as i16, *z as i16);
                self.unit_coordinator.order_move(target);
                true
            }
            GameCommand::ToggleSimulation => {
                if self.game_world.state == GameState::InGame {
                    self.game_world.state = GameState::Frontend;
                    log::info!("game simulation OFF");
                } else {
                    self.game_world.state = GameState::InGame;
                    log::info!("game simulation ON");
                }
                true
            }
            GameCommand::IncreaseGameSpeed => {
                let new_speed = (self.game_world.game_speed + 2).min(30);
                self.game_world.set_game_speed(new_speed);
                println!("game speed: {} ticks/sec", self.game_world.game_speed);
                false
            }
            GameCommand::DecreaseGameSpeed => {
                let new_speed = self.game_world.game_speed.saturating_sub(2).max(4);
                self.game_world.set_game_speed(new_speed);
                println!("game speed: {} ticks/sec", self.game_world.game_speed);
                false
            }
            GameCommand::SetHudTab(tab) => {
                self.hud_tab = *tab;
                true
            }
            GameCommand::ToggleHud => {
                self.hud_visible = !self.hud_visible;
                true
            }
            GameCommand::ToggleCompass => {
                self.compass_visible = !self.compass_visible;
                true
            }
            GameCommand::ToggleWalkability => {
                self.walkability_visible = !self.walkability_visible;
                true
            }
            GameCommand::Quit => true,
            // Building commands: handled by game logic, not renderer
            GameCommand::PlaceBuilding { .. }
            | GameCommand::CancelPlacement
            | GameCommand::EnterBuildMode { .. }
            | GameCommand::EnterBuilding { .. }
            | GameCommand::TrainUnit { .. } => false,
        }
    }

    /// Produce the output boundary for the renderer — a snapshot of all
    /// game-logic state needed to draw one frame.
    fn frame_state<'a>(&'a self, drag_state: &'a DragState) -> FrameState<'a> {
        FrameState {
            camera: &self.camera,
            screen: &self.screen,
            zoom: self.zoom,
            landscape: &self.landscape_mesh,
            curvature_scale: if self.curvature_enabled { self.curvature_scale } else { 0.0 },
            sunlight: self.sunlight,
            wat_offset: self.wat_offset,
            show_objects: self.show_objects,
            show_shadows: self.show_shadows,
            show_lighting: self.show_lighting,
            show_markers: self.show_markers,
            unit_coordinator: &self.unit_coordinator,
            level_objects: &self.level_objects,
            building_objects: &self.building_objects,
            scenery_objects: &self.scenery_objects,
            shapes: &self.shapes,
            hud_state: self.build_hud_state(),
            drag_state,
            ghost_preview: None,
            needs_building_rebuild: false,
            needs_spawn_rebuild: false,
            needs_unit_rebuild: false,
            needs_level_reload: false,
        }
    }
}

pub struct App {
    engine: GameEngine,
    input: InputState,

    // Window / GPU
    window: Option<Arc<Window>>,
    gpu: Option<GpuContext>,

    // Landscape rendering
    program_container: LandscapeProgramContainer,
    landscape_group0_layout: Option<wgpu::BindGroupLayout>,
    landscape_group0_bind_group: Option<wgpu::BindGroup>,
    model_main: Option<ModelEnvelop<LandscapeModel>>,

    // Object marker bind groups (shared by markers, unit markers, selection rings)
    objects_group0_bind_group: Option<wgpu::BindGroup>,
    objects_group1_bind_group: Option<wgpu::BindGroup>,

    // Person unit sprites (per-type atlas + model)
    spawn_pipeline: Option<wgpu::RenderPipeline>,
    sprite_group1_layout: Option<wgpu::BindGroupLayout>,
    unit_renders: Vec<UnitTypeRender>,

    // Level object markers
    objects_marker_pipeline: Option<wgpu::RenderPipeline>,
    model_objects: Option<ModelEnvelop<ColorModel>>,

    // Shadow mapping
    shadow_depth_view: Option<wgpu::TextureView>,
    shadow_depth_building_pipeline: Option<wgpu::RenderPipeline>,

    shadow_pass_group0: Option<wgpu::BindGroup>,
    light_mvp_buffer: Option<GpuBuffer>,
    shadow_recv_group2_layout: Option<wgpu::BindGroupLayout>,
    shadow_recv_group2: Option<wgpu::BindGroup>,

    // Lighting
    building_bind_group_0: Option<wgpu::BindGroup>,
    lighting_buffer: Option<GpuBuffer>,

    // 3D building meshes
    building_pipeline: Option<wgpu::RenderPipeline>,
    building_bind_group_1: Option<wgpu::BindGroup>,
    model_buildings: Option<ModelEnvelop<TexModel>>,

    // Sky
    sky_pipeline: Option<wgpu::RenderPipeline>,
    sky_bind_group: Option<wgpu::BindGroup>,
    sky_uniform_buffer: Option<GpuBuffer>,

    // HUD renderer
    hud: Option<HudRenderer>,

    // Shared uniform buffers
    mvp_buffer: Option<GpuBuffer>,
    model_transform_buffer: Option<GpuBuffer>,
    landscape_params_buffer: Option<GpuBuffer>,
    select_params_buffer: Option<GpuBuffer>,

    // Storage buffers (level-dependent)
    heights_buffer: Option<GpuBuffer>,
    watdisp_buffer: Option<GpuBuffer>,

    // Unit rendering
    model_unit_markers: Option<ModelEnvelop<ColorModel>>,
    model_selection_outlines: Option<ModelEnvelop<ColorModel>>,
    model_walkability: Option<ModelEnvelop<ColorModel>>,
    walkability_pipeline: Option<wgpu::RenderPipeline>,

    // Render flag
    do_render: bool,

    // Debug logging
    debug_log: BufWriter<File>,
    start_time: Instant,

    // Script replay
    script_commands: Vec<String>,
    script_index: usize,

    // Smooth camera pan to shaman
    shaman_pan: Option<ShamanPanAnimation>,

    // Screenshot capture
    screenshot_path: Option<String>,
    screenshot_counter: u32,
    // Script wait (wall-clock based)
    script_wait_until: Option<Instant>,
}

struct ShamanPanAnimation {
    start_shift: (usize, usize),
    target_shift: (usize, usize),
    start_time: Instant,
    duration: f32,
}

/// Shortest signed delta from `from` to `to` on a toroidal axis of size `n`.
fn toroidal_delta(from: usize, to: usize, n: usize) -> i32 {
    let d = (to as i32 - from as i32).rem_euclid(n as i32);
    if d <= (n as i32) / 2 { d } else { d - n as i32 }
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        let camera = Camera::new();

        let sunlight = {
            let (x, y) = config.light.unwrap_or((0x93, 0x93));
            Vector4::<f32>::new(x as f32, y as f32, 0x93 as f32, 0.0)
        };

        let landscape_mesh = LandscapeMesh::new(1.0 / 16.0, (1.0 / 16.0) * 4.0 / 1024.0);

        let debug_log = BufWriter::new(
            File::create("/tmp/pop3_debug.jsonl").expect("failed to create debug log"),
        );

        let script_commands: Vec<String> = config.script.as_ref()
            .map(|path| {
                std::fs::read_to_string(path)
                    .unwrap_or_else(|e| panic!("failed to read script {:?}: {}", path, e))
                    .lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty() && !l.starts_with('#'))
                    .collect()
            })
            .unwrap_or_default();

        let level_num = config.level.unwrap_or(1);

        let mut app = App {
            engine: GameEngine {
                landscape_mesh,
                camera,
                screen: Screen { width: 800, height: 600 },
                curvature_scale: 0.0512,
                curvature_enabled: true,
                zoom: 1.0,
                level_num,
                sunlight,
                show_objects: true,
                show_shadows: true,
                show_lighting: true,
                show_markers: false,
                sprite_z_offset: 0.005,
                sprite_scale: 0.65,
                hud_tab: HudTab::Spells,
                hud_visible: false,
                compass_visible: false,
                walkability_visible: false,
                hud_panel_sprite_count: 0,
                unit_coordinator: UnitCoordinator::new(),
                game_world: {
                    let mut w = GameWorld::new(20);
                    w.state = GameState::InGame;
                    w
                },
                game_time: StdTimeSource::new(),
                level_objects: Vec::new(),
                building_objects: Vec::new(),
                scenery_objects: Vec::new(),
                shapes: Vec::new(),
                shape_footprints: ShapeFootprints::empty(),
                wat_offset: -1,
                wat_interval: 5000,
                frame_count: 0,
                config,
            },
            input: InputState {
                mouse_pos: Point2::<f32>::new(0.0, 0.0),
                drag_state: DragState::None,
            },
            window: None,
            gpu: None,
            program_container: LandscapeProgramContainer::new(),
            landscape_group0_layout: None,
            landscape_group0_bind_group: None,
            model_main: None,
            objects_group0_bind_group: None,
            objects_group1_bind_group: None,
            spawn_pipeline: None,
            sprite_group1_layout: None,
            unit_renders: Vec::new(),
            shadow_depth_view: None,
            shadow_depth_building_pipeline: None,

            shadow_pass_group0: None,
            light_mvp_buffer: None,
            shadow_recv_group2_layout: None,
            shadow_recv_group2: None,
            building_bind_group_0: None,
            lighting_buffer: None,
            objects_marker_pipeline: None,
            model_objects: None,
            building_pipeline: None,
            building_bind_group_1: None,
            model_buildings: None,
            sky_pipeline: None,
            sky_bind_group: None,
            sky_uniform_buffer: None,
            hud: None,
            mvp_buffer: None,
            model_transform_buffer: None,
            landscape_params_buffer: None,
            select_params_buffer: None,
            heights_buffer: None,
            watdisp_buffer: None,
            model_unit_markers: None,
            model_selection_outlines: None,
            model_walkability: None,
            walkability_pipeline: None,
            do_render: true,
            debug_log,
            start_time: Instant::now(),
            script_commands,
            script_index: 0,
            shaman_pan: None,
            screenshot_path: None,
            screenshot_counter: 0,
            script_wait_until: None,
        };
        app.engine.reset_camera();
        app
    }

    fn center_on_tribe0_shaman(&mut self) {
        let shaman_cell = self.unit_renders.iter()
            .find(|ur| ur.subtype == PERSON_SUBTYPE_SHAMAN)
            .and_then(|ur| ur.cells.iter().find(|c| c.tribe_index == 0));
        let shaman_pos = shaman_cell.map(|c| (c.cell_x, c.cell_y));
        if let Some((cx, cy)) = shaman_pos {
            let n = self.engine.landscape_mesh.width() as i32;
            let v = self.engine.camera_focus_vertex() as i32;
            let target_sx = ((cx as i32 - v) % n + n) % n;
            let target_sy = ((cy as i32 - v) % n + n) % n;

            let cur = self.engine.landscape_mesh.get_shift_vector();
            let cur_sx = cur.x as usize;
            let cur_sy = cur.y as usize;

            log::info!("[center] shaman at cell ({}, {}), pan ({},{}) -> ({},{})",
                cx, cy, cur_sx, cur_sy, target_sx, target_sy);

            self.shaman_pan = Some(ShamanPanAnimation {
                start_shift: (cur_sx, cur_sy),
                target_shift: (target_sx as usize, target_sy as usize),
                start_time: Instant::now(),
                duration: 0.5,
            });
        } else {
            log::warn!("[center] no tribe 0 shaman in unit_renders");
        }
    }

    fn tick_shaman_pan(&mut self) {
        let Some(anim) = &self.shaman_pan else { return };

        let n = self.engine.landscape_mesh.width();
        let elapsed = anim.start_time.elapsed().as_secs_f32();
        let t = (elapsed / anim.duration).clamp(0.0, 1.0);
        let s = t * t * (3.0 - 2.0 * t); // smoothstep

        let (sx0, sy0) = anim.start_shift;
        let (sxt, syt) = anim.target_shift;
        let dx = toroidal_delta(sx0, sxt, n);
        let dy = toroidal_delta(sy0, syt, n);

        let new_sx = ((sx0 as i32 + (dx as f32 * s).round() as i32).rem_euclid(n as i32)) as usize;
        let new_sy = ((sy0 as i32 + (dy as f32 * s).round() as i32).rem_euclid(n as i32)) as usize;

        let old = self.engine.landscape_mesh.get_shift_vector();
        self.engine.landscape_mesh.set_shift(new_sx, new_sy);
        let new = self.engine.landscape_mesh.get_shift_vector();

        if old != new {
            self.rebuild_spawn_model();
            self.do_render = true;
        }

        if t >= 1.0 {
            self.shaman_pan = None;
        }
    }

    fn update_level(&mut self) {
        self.shaman_pan = None;
        self.engine.reset_camera();
        let base = self.engine.config.base.clone().unwrap_or_else(|| Path::new("/opt/sandbox/pop").to_path_buf());
        let level_type = self.engine.config.landtype.as_deref();
        let level_res = LevelRes::new(&base, self.engine.level_num, level_type);

        let shores = level_res.landscape.make_shores();
        self.engine.landscape_mesh.set_heights(&shores.height);

        {
            let gpu = self.gpu.as_ref().unwrap();

            // Update heights buffer
            let heights_vec = shores.to_vec();
            let heights_bytes: &[u8] = bytemuck::cast_slice(&heights_vec);
            let heights_buffer = GpuBuffer::new_storage(&gpu.device, heights_bytes, "heights_buffer");
            self.heights_buffer = Some(heights_buffer);

            // Update watdisp buffer
            let watdisp_vec: Vec<u32> = level_res.params.watdisp.iter().map(|v| *v as u32).collect();
            let watdisp_bytes: &[u8] = bytemuck::cast_slice(&watdisp_vec);
            let watdisp_buffer = GpuBuffer::new_storage(&gpu.device, watdisp_bytes, "watdisp_buffer");
            self.watdisp_buffer = Some(watdisp_buffer);
        }

        // Rebuild all landscape variants
        self.rebuild_landscape_variants(&level_res);

        // Rebuild per-unit-type sprite atlases with new palette
        self.rebuild_unit_atlases(&base, &level_res.params.palette);

        // Rebuild unit cells and object markers
        self.engine.level_objects = extract_level_objects(&level_res);

        // Extract person units into the coordinator (they become live entities)
        self.engine.unit_coordinator.load_level(&level_res.units, &shores.height, level_res.landscape.land_size());
        // Remove persons from static markers — they're now rendered by the coordinator
        self.engine.level_objects.retain(|obj| obj.model_type != ModelType::Person);

        // Populate unit_renders cells from live coordinator units
        self.sync_unit_render_cells();

        // Flatten terrain under buildings (modifies heightmap + re-uploads GPU buffer)
        self.flatten_terrain_under_buildings();

        // Mark building footprints in region map for pathfinding walkability
        self.populate_buildings_in_region_map();

        self.rebuild_spawn_model();
        self.center_on_tribe0_shaman();

        // Rebuild HUD atlas with new palette
        let panel_path = base.join("data").join("plspanel.spr");
        if let Some(panel_container) = ContainerPSFB::from_file(&panel_path) {
            self.engine.hud_panel_sprite_count = panel_container.len();
            if let Some(ref mut hud) = self.hud {
                let gpu = self.gpu.as_ref().unwrap();
                hud.build_atlas(&gpu.device, &gpu.queue, &panel_container, &level_res.params.palette);
            }
        }
    }

    fn log_camera_state(&mut self, event: &str) {
        let t = self.start_time.elapsed().as_secs_f64();
        let center = self.engine.world_center();
        let az = (self.engine.camera.angle_z as f32).to_radians();
        let ax = (self.engine.camera.angle_x as f32).to_radians();
        let radius = 1.5 / self.engine.zoom;
        let eye_x = center + radius * ax.cos() * az.sin();
        let eye_y = center + radius * ax.cos() * az.cos();
        let eye_z_orbit = -radius * ax.sin();
        let min_z = self.engine.camera_min_z();
        let eye_z = eye_z_orbit.max(min_z);
        let shift = self.engine.landscape_mesh.get_shift_vector();
        let _ = writeln!(
            self.debug_log,
            r#"{{"t":{:.3},"event":"{}","angle_x":{},"angle_z":{},"zoom":{:.3},"radius":{:.4},"eye":[{:.4},{:.4},{:.4}],"eye_z_orbit":{:.4},"min_z":{:.4},"focus":[{:.4},{:.4},0.0],"shift":[{},{}]}}"#,
            t, event,
            self.engine.camera.angle_x, self.engine.camera.angle_z,
            self.engine.zoom, radius,
            eye_x, eye_y, eye_z, eye_z_orbit, min_z,
            center, center,
            shift.x, shift.y,
        );
        let _ = self.debug_log.flush();
    }

    fn is_script_mode(&self) -> bool {
        !self.script_commands.is_empty()
    }

    fn run_script_step(&mut self) -> bool {
        if self.script_index >= self.script_commands.len() {
            return false; // done
        }
        // If we're in a timed wait, check if it's expired
        if let Some(deadline) = self.script_wait_until {
            if Instant::now() < deadline {
                self.do_render = true;
                return true; // keep waiting
            }
            self.script_wait_until = None;
        }

        let cmd = self.script_commands[self.script_index].clone();
        self.script_index += 1;

        // Parse wait command: "wait N" — pause for N seconds (wall-clock)
        if let Some(val) = cmd.strip_prefix("wait ") {
            if let Ok(secs) = val.trim().parse::<f32>() {
                self.script_wait_until = Some(Instant::now() + Duration::from_secs_f32(secs));
                self.do_render = true;
                return true;
            }
        }

        // Parse click command: "click X Y" — left-click at screen position
        if let Some(coords) = cmd.strip_prefix("click ") {
            let parts: Vec<&str> = coords.trim().split_whitespace().collect();
            if parts.len() == 2 {
                if let (Ok(x), Ok(y)) = (parts[0].parse::<f32>(), parts[1].parse::<f32>()) {
                    self.input.mouse_pos = Point2::new(x, y);
                    log::info!("[script] click at ({}, {})", x, y);
                    // Simulate left press + release (selection)
                    match self.engine.find_unit_at_screen_pos(&self.input.mouse_pos) {
                        Some(id) => {
                            self.engine.unit_coordinator.selection.select_single(id);
                            log::info!("[script] selected unit {}", id);
                        }
                        None => {
                            self.engine.unit_coordinator.selection.clear();
                            log::info!("[script] no unit at click, selection cleared");
                        }
                    }
                    self.rebuild_unit_models();
                    self.do_render = true;
                    return true;
                }
            }
        }

        // Parse rightclick command: "rightclick X Y" — right-click move order
        if let Some(coords) = cmd.strip_prefix("rightclick ") {
            let parts: Vec<&str> = coords.trim().split_whitespace().collect();
            if parts.len() == 2 {
                if let (Ok(x), Ok(y)) = (parts[0].parse::<f32>(), parts[1].parse::<f32>()) {
                    self.input.mouse_pos = Point2::new(x, y);
                    log::info!("[script] rightclick at ({}, {})", x, y);
                    if let Some((cx, cy)) = self.engine.screen_to_cell(&self.input.mouse_pos) {
                        let target = cell_to_world(cx, cy, self.engine.landscape_mesh.width() as f32);
                        let walkable = self.engine.unit_coordinator.region_map().is_walkable(target.to_tile());
                        log::info!("[script] rightclick cell=({:.1}, {:.1}) → world=({}, {}) walkable={}",
                            cx, cy, target.x, target.z, walkable);
                        self.engine.apply_command(&GameCommand::OrderMove {
                            x: target.x as f32, z: target.z as f32,
                        });
                    } else {
                        log::warn!("[script] rightclick: screen_to_cell returned None");
                    }
                    self.do_render = true;
                    return true;
                }
            }
        }

        // Parse dump command: log all unit screen positions
        if cmd.trim() == "dump_units" {
            let pvm = self.engine.unit_pvm();
            for unit in self.engine.unit_coordinator.units() {
                if let Some((sx, sy)) = self.engine.unit_screen_pos(unit, &pvm) {
                    log::info!("[dump] unit {} tribe={} cell=({:.2}, {:.2}) screen=({:.0}, {:.0})",
                        unit.id, unit.tribe_index, unit.cell_x, unit.cell_y, sx, sy);
                } else {
                    log::info!("[dump] unit {} tribe={} cell=({:.2}, {:.2}) behind camera",
                        unit.id, unit.tribe_index, unit.cell_x, unit.cell_y);
                }
            }
            return true;
        }

        // Parse dump_buildings command: log footprint coordinate details
        if cmd.trim() == "dump_buildings" {
            self.dump_building_footprints();
            return true;
        }

        // Parse screenshot command: "screenshot [path]"
        if let Some(path) = cmd.strip_prefix("screenshot ") {
            self.screenshot_path = Some(path.trim().to_string());
            self.do_render = true;
            return true;
        }
        if cmd.trim() == "screenshot" {
            let path = format!("screenshot_{:04}.png", self.screenshot_counter);
            self.screenshot_counter += 1;
            self.screenshot_path = Some(path);
            self.do_render = true;
            return true;
        }

        // Parse zoom command
        if let Some(val) = cmd.strip_prefix("zoom ") {
            if let Ok(z) = val.trim().parse::<f32>() {
                self.engine.apply_command(&GameCommand::SetZoom(z));
                self.log_camera_state("zoom");
                self.do_render = true;
                return true;
            }
        }

        // Parse key name to KeyCode
        let key = match cmd.as_str() {
            "W" => KeyCode::KeyW,
            "A" => KeyCode::KeyA,
            "S" => KeyCode::KeyS,
            "D" => KeyCode::KeyD,
            "Q" => KeyCode::KeyQ,
            "E" => KeyCode::KeyE,
            "R" => KeyCode::KeyR,
            "T" => KeyCode::KeyT,
            "N" => KeyCode::KeyN,
            "M" => KeyCode::KeyM,
            "B" => KeyCode::KeyB,
            "V" => KeyCode::KeyV,
            "C" => KeyCode::KeyC,
            "Space" => KeyCode::Space,
            "ArrowUp" => KeyCode::ArrowUp,
            "ArrowDown" => KeyCode::ArrowDown,
            "BracketLeft" => KeyCode::BracketLeft,
            "BracketRight" => KeyCode::BracketRight,
            "F8" => KeyCode::F8,
            "Escape" => KeyCode::Escape,
            other => {
                log::warn!("script: unknown command {:?}", other);
                return true; // skip, continue
            }
        };

        // Replay through translate_key → apply_command
        if let Some(cmd) = translate_key(key) {
            let prev_shift = self.engine.landscape_mesh.get_shift_vector();
            self.engine.apply_command(&cmd);

            // App-level side effects (same as keyboard handler)
            match &cmd {
                GameCommand::Quit => { return false; }
                GameCommand::NextShader => { self.program_container.next(); }
                GameCommand::PrevShader => { self.program_container.prev(); }
                GameCommand::NextLevel | GameCommand::PrevLevel => {
                    self.update_level();
                }
                GameCommand::CenterOnShaman => {
                    self.center_on_tribe0_shaman();
                    self.log_camera_state("space_center");
                }
                GameCommand::ResetCamera => {
                    self.rebuild_spawn_model();
                    self.log_camera_state("reset");
                }
                GameCommand::TopDownView => {
                    self.log_camera_state("KeyT");
                }
                GameCommand::ToggleCurvature | GameCommand::AdjustCurvature { .. }
                | GameCommand::AdjustSpriteOffset { .. } | GameCommand::AdjustSpriteScale { .. } => {
                    self.rebuild_spawn_model();
                }
                GameCommand::PanScreen { .. } | GameCommand::PanTerrain { .. } => {
                    self.shaman_pan = None;
                    let new_shift = self.engine.landscape_mesh.get_shift_vector();
                    if new_shift != prev_shift {
                        self.rebuild_spawn_model();
                        self.log_camera_state(&format!("{:?}", key));
                    }
                }
                GameCommand::RotateCamera { .. } | GameCommand::TiltCamera { .. } => {
                    self.rebuild_spawn_model();
                    self.log_camera_state(&format!("{:?}", key));
                }
                _ => {}
            }
            self.do_render = true;
        }
        true
    }

    /// Sync unit_renders cells from live coordinator units.
    fn sync_unit_render_cells(&mut self) {
        use crate::render::sprites::UnitRenderData;
        for ur in &mut self.unit_renders {
            ur.cells.clear();
        }
        for unit in self.engine.unit_coordinator.units() {
            if !unit.alive { continue; }
            if let Some(ur) = self.unit_renders.iter_mut().find(|u| u.subtype == unit.subtype) {
                ur.cells.push(UnitRenderData {
                    cell_x: unit.cell_x,
                    cell_y: unit.cell_y,
                    tribe_index: unit.tribe_index,
                    facing_angle: unit.movement.facing_angle,
                    frame_index: unit.anim.frame_index,
                    animation_id: unit.anim.animation_id,
                });
            }
        }
    }

    fn rebuild_spawn_model(&mut self) {
        if let Some(ref gpu) = self.gpu {
            let cs = if self.engine.curvature_enabled { self.engine.curvature_scale } else { 0.0 };
            for ur in &mut self.unit_renders {
                if !ur.cells.is_empty() {
                    ur.model = Some(build_spawn_model(
                        &gpu.device, &ur.cells, &self.engine.landscape_mesh, cs,
                        self.engine.camera.angle_x, self.engine.camera.angle_z,
                        ur.frame_width, ur.frame_height, ur.frames_per_dir,
                        &ur.anim_offsets,
                        self.engine.sprite_z_offset, self.engine.sprite_scale,
                    ));
                } else {
                    ur.model = None;
                }
            }
        }
        self.rebuild_object_markers();
    }

    /// Rebuild all unit type atlases (e.g. on level change with new palette).
    fn rebuild_unit_atlases(&mut self, base: &Path, raw_palette: &[u8]) {
        let gpu = match self.gpu.as_ref() {
            Some(g) => g,
            None => return,
        };
        let layout = match self.sprite_group1_layout.as_ref() {
            Some(l) => l,
            None => return,
        };

        let palette = convert_palette(raw_palette);
        let hspr_path = base.join("data").join("HSPR0-0.DAT");
        let container = match ContainerPSFB::from_file(&hspr_path) {
            Some(c) => c,
            None => return,
        };
        let anim_data = AnimationsData::from_path(&base.join("data"));
        let sequences = AnimationSequence::from_data(&anim_data);
        let sampler = GpuTexture::create_sampler(&gpu.device, true);

        // Extract frame counts per animation ID (using shape table indirection).
        // Animation IDs map through ANIM_SHAPE_TABLE to VSTART bases;
        // frame_count = max frames across the 5 stored directions.
        use crate::data::animation::{STORED_DIRECTIONS, anim_shape, ANIM_SHAPE_TABLE};
        let num_anim_ids = ANIM_SHAPE_TABLE.len();
        let mut frame_counts = vec![1u8; num_anim_ids + 1];
        for anim_id in 0..num_anim_ids {
            let (vstart_base, _sprite_type) = anim_shape(anim_id as u16);
            let mut max_frames = 0usize;
            for dir in 0..STORED_DIRECTIONS {
                let seq_idx = vstart_base + dir;
                if seq_idx < sequences.len() {
                    max_frames = max_frames.max(sequences[seq_idx].frames.len());
                }
            }
            frame_counts[anim_id] = max_frames.min(255) as u8;
        }
        self.engine.unit_coordinator.anim_frame_counts = frame_counts;

        self.unit_renders.clear();

        // Non-shaman subtypes: build combined idle+walk atlas
        for &(subtype, anim_indices) in &UNIT_MULTI_ANIMS {
            if let Some((atlas_w, atlas_h, rgba, fw, fh, total_cols, offsets, _max_y)) =
                build_multi_anim_atlas(&sequences, &container, &palette, anim_indices)
            {
                let tex = GpuTexture::new_2d(
                    &gpu.device, &gpu.queue, atlas_w, atlas_h,
                    wgpu::TextureFormat::Rgba8UnormSrgb, &rgba,
                    &format!("unit_atlas_st{}", subtype),
                );
                let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(&format!("unit_bg1_st{}", subtype)),
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&tex.view) },
                        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
                    ],
                });
                let anim_offsets: Vec<(u16, u32, u32)> = offsets.iter()
                    .map(|(idx, off, fc)| (*idx as u16, *off, *fc))
                    .collect();
                self.unit_renders.push(UnitTypeRender {
                    subtype,
                    cells: Vec::new(),
                    texture: tex,
                    bind_group,
                    model: None,

                    frame_width: fw,
                    frame_height: fh,
                    frames_per_dir: total_cols,
                    anim_offsets,
                });
            }
        }

        // Shaman: pre-rendered per-tribe sprites (not VELE composited)
        {
            let subtype = PERSON_SUBTYPE_SHAMAN;
            if let Some((atlas_w, atlas_h, rgba, fw, fh, total_cols, offsets, _max_y)) =
                build_direct_multi_anim_atlas(&container, &palette, &SHAMAN_ANIMS)
            {
                let tex = GpuTexture::new_2d(
                    &gpu.device, &gpu.queue, atlas_w, atlas_h,
                    wgpu::TextureFormat::Rgba8UnormSrgb, &rgba,
                    &format!("unit_atlas_st{}", subtype),
                );
                let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(&format!("unit_bg1_st{}", subtype)),
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&tex.view) },
                        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
                    ],
                });
                let anim_offsets: Vec<(u16, u32, u32)> = offsets.iter()
                    .map(|(idx, off, fc)| (*idx as u16, *off, *fc))
                    .collect();
                self.unit_renders.push(UnitTypeRender {
                    subtype,
                    cells: Vec::new(),
                    texture: tex,
                    bind_group,
                    model: None,

                    frame_width: fw,
                    frame_height: fh,
                    frames_per_dir: total_cols,
                    anim_offsets,
                });
            }
        }
    }

    fn rebuild_unit_models(&mut self) {
        if let Some(ref gpu) = self.gpu {
            let cs = if self.engine.curvature_enabled { self.engine.curvature_scale } else { 0.0 };
            self.model_unit_markers = build_unit_markers(
                &gpu.device, self.engine.unit_coordinator.units(), &self.engine.landscape_mesh, cs,
                self.engine.camera.angle_x, self.engine.camera.angle_z,
            );
            self.model_selection_outlines = build_selection_outlines(
                &gpu.device, &self.engine.unit_coordinator,
                &self.engine.landscape_mesh, cs,
                self.engine.camera.angle_x, self.engine.camera.angle_z,
            );
        }
    }

    /// Flatten terrain under all building footprints (matching original game's
    /// Building_FlattenTerrain @ 0x0042F2A0). Modifies self.engine.landscape_mesh heights
    /// and re-uploads the GPU heights buffer.
    /// Look up the OBJS footprint index for a level object.
    /// Returns the SHAPES.DAT index from the OBJS entry's fp_idx[rotation].
    fn obj_footprint_idx(&self, obj: &LevelObject) -> Option<usize> {
        let idx = object_3d_index(&obj.model_type, obj.subtype, obj.tribe_index)?;
        // Use the correct OBJS bank: bank 0 for buildings, level bank for scenery
        let bank = match obj.model_type {
            ModelType::Scenery => &self.engine.scenery_objects,
            _ => &self.engine.building_objects,
        };
        let obj3d = bank.get(idx)?.as_ref()?;
        let fp = obj3d.footprint_index(0); // always use rotation 0 (base shape)
        if fp < 0 || (fp as usize) >= self.engine.shapes.len() { return None; }
        Some(fp as usize)
    }

    fn flatten_terrain_under_buildings(&mut self) {
        // Collect building info first to avoid borrow conflicts
        let buildings: Vec<_> = self.engine.level_objects.iter()
            .filter(|obj| obj.model_type == ModelType::Building)
            .filter_map(|obj| {
                let fp_idx = self.obj_footprint_idx(obj)?;
                Some((obj.cell_x as i32, obj.cell_y as i32, fp_idx))
            })
            .collect();
        for (cx, cy, fp_idx) in buildings {
            let shape = self.engine.shapes[fp_idx];
            self.engine.landscape_mesh.flatten_building_footprint(
                cx, cy, &shape, fp_idx,
                &self.engine.shape_footprints,
                true,
            );
        }

        // Re-upload modified heights to GPU
        if let Some(ref gpu) = self.gpu {
            if let Some(ref heights_buf) = self.heights_buffer {
                let heights_vec = self.engine.landscape_mesh.heights_to_gpu_vec();
                let heights_bytes: &[u8] = bytemuck::cast_slice(&heights_vec);
                heights_buf.update(&gpu.queue, 0, heights_bytes);
            }
        }
    }

    fn populate_buildings_in_region_map(&mut self) {
        let n = self.engine.landscape_mesh.width();
        let ni = n as i32;
        // terrain class 2 = building = unwalkable (matches original binary)
        self.engine.unit_coordinator.region_map_mut().set_terrain_flags(2, 0x00);
        for obj in &self.engine.level_objects {
            if obj.model_type != ModelType::Building && obj.model_type != ModelType::Scenery { continue; }
            let fp_idx = match self.obj_footprint_idx(obj) {
                Some(i) => i,
                None => continue,
            };
            let shape = &self.engine.shapes[fp_idx];
            let w = shape.width as i32;
            let h = shape.height as i32;
            // Origin is in tile units (2 per cell), convert to cell units
            let ox = shape.origin_x as i32 / 2;
            let oz = shape.origin_z as i32 / 2;
            let base_cx = obj.cell_x as i32 - ox;
            let base_cy = obj.cell_y as i32 - oz;
            let mut marked = 0u32;
            for dy in 0..h {
                for dx in 0..w {
                    if self.engine.shape_footprints.is_cell_occupied(fp_idx, dx as usize, dy as usize) {
                        let cx = ((base_cx + dx) % ni + ni) % ni;
                        let cy = ((base_cy + dy) % ni + ni) % ni;
                        let tile = cell_to_tile(cx, cy, ni);
                        let cell = self.engine.unit_coordinator.region_map_mut().get_cell_mut(tile);
                        cell.terrain_type = 2;
                        cell.flags_high |= CELL_HAS_BUILDING;
                        marked += 1;
                    }
                }
            }
            log::info!("[footprint] {:?} subtype={} cell=({},{}) fp_idx={} shape={}x{} marked={}",
                obj.model_type, obj.subtype, obj.cell_x, obj.cell_y,
                fp_idx, w, h, marked);
        }
    }

    fn dump_building_footprints(&self) {
        let n = self.engine.landscape_mesh.width();
        let ni = n as i32;

        // 1) Check tile-mapping uniqueness: do all 128×128 cells map to unique tiles?
        let mut seen = std::collections::HashSet::new();
        let mut duplicates = 0u32;
        for cy in 0..ni {
            for cx in 0..ni {
                let tile = cell_to_tile(cx, cy, ni);
                let idx = tile.cell_index();
                if !seen.insert(idx) {
                    duplicates += 1;
                    if duplicates <= 5 {
                        log::warn!("[tile-map] DUPLICATE: cell ({},{}) → tile ({},{}) → idx {}",
                            cx, cy, tile.x, tile.z, idx);
                    }
                }
            }
        }
        log::info!("[tile-map] {} unique tiles out of {} cells, {} duplicates",
            seen.len(), n * n, duplicates);

        // 2) Dump each building/scenery footprint cells
        let ni = n as i32;
        for obj in &self.engine.level_objects {
            if obj.model_type != ModelType::Building && obj.model_type != ModelType::Scenery { continue; }
            let fp_idx = match self.obj_footprint_idx(obj) {
                Some(idx) => idx,
                None => continue,
            };
            let shape = &self.engine.shapes[fp_idx];
            let w = shape.width as i32;
            let h = shape.height as i32;
            // Origin is in tile units (2 per cell), convert to cell units
            let ox = shape.origin_x as i32 / 2;
            let oz = shape.origin_z as i32 / 2;
            let base_cx = obj.cell_x as i32 - ox;
            let base_cy = obj.cell_y as i32 - oz;

            log::info!("[footprint] {:?} subtype={} cell=({:.1},{:.1}) corner=({},{}) shape={}x{} origin=({},{}) fp_idx={}",
                obj.model_type, obj.subtype, obj.cell_x, obj.cell_y, base_cx, base_cy, w, h,
                shape.origin_x, shape.origin_z, fp_idx);

            for dy in 0..h {
                for dx in 0..w {
                    let occupied = self.engine.shape_footprints.is_cell_occupied(fp_idx, dx as usize, dy as usize);
                    let cx = ((base_cx + dx) % ni + ni) % ni;
                    let cy = ((base_cy + dy) % ni + ni) % ni;
                    let tile = cell_to_tile(cx, cy, ni);
                    let cell_idx = tile.cell_index();
                    let region_cell = self.engine.unit_coordinator.region_map().get_cell(tile);
                    log::info!("[footprint]   dx={} dy={} occ={} → cell ({},{}) → tile ({},{}) idx={} rm_type={} rm_bldg={}",
                        dx, dy, occupied, cx, cy,
                        tile.x, tile.z, cell_idx,
                        region_cell.terrain_type, region_cell.has_building());
                }
            }
        }
    }

    fn rebuild_object_markers(&mut self) {
        if let Some(ref gpu) = self.gpu {
            let cs = if self.engine.curvature_enabled { self.engine.curvature_scale } else { 0.0 };
            self.model_objects = Some(build_object_markers(
                &gpu.device, &self.engine.level_objects, &self.engine.landscape_mesh, cs,
                self.engine.camera.angle_x, self.engine.camera.angle_z,
            ));
            self.model_buildings = Some(build_building_meshes(
                &gpu.device, &self.engine.level_objects,
                &self.engine.building_objects, &self.engine.scenery_objects,
                &self.engine.shapes, &self.engine.landscape_mesh, cs,
            ));
        }
        self.rebuild_unit_models();
        self.rebuild_walkability_overlay();
    }

    fn rebuild_walkability_overlay(&mut self) {
        let gpu = match self.gpu.as_ref() {
            Some(g) => g,
            None => return,
        };
        let region_map = self.engine.unit_coordinator.region_map();
        let landscape = &self.engine.landscape_mesh;
        let step = landscape.step();
        let w = landscape.width();
        let wf = w as f32;
        let shift = landscape.get_shift_vector();
        let center = (wf - 1.0) * step / 2.0;
        let cs = if self.engine.curvature_enabled { self.engine.curvature_scale } else { 0.0 };

        let walkable_color = Vector3::new(0.0, 0.8, 0.2);   // green = walkable land
        let building_color = Vector3::new(0.8, 0.3, 0.1);   // red-brown = building
        let water_color = Vector3::new(0.0, 0.3, 0.8);      // blue = water
        let shore_color = Vector3::new(0.6, 0.6, 0.1);      // yellow-brown = shore buffer (unwalkable)
        let height_offset = 0.02;

        let mut model: ColorModel = MeshModel::new();
        let mut count_water = 0u32;
        let mut count_building = 0u32;
        let mut count_walkable = 0u32;

        let heights = landscape.heights();

        for cell_y in 0..w {
            for cell_x in 0..w {
                // Check walkability via region map
                let tile = cell_to_tile(cell_x as i32, cell_y as i32, w as i32);
                let is_building = region_map.has_building(tile);

                // Look up 4 corner heights to determine per-triangle water status
                let cx1 = (cell_x + 1) % w;
                let cy1 = (cell_y + 1) % w;
                let h00 = heights[cell_y][cell_x];   // (x, y)     = TL
                let h10 = heights[cell_y][cx1];       // (x+1, y)   = TR
                let h01 = heights[cy1][cell_x];       // (x, y+1)   = BL
                let h11 = heights[cy1][cx1];           // (x+1, y+1) = BR

                // "/" diagonal split matching terrain mesh:
                //   Triangle A (lower-left):  TL(0,0) - BL(0,1) - TR(1,0)
                //   Triangle B (upper-right): BR(1,1) - BL(0,1) - TR(1,0)
                let tri_a_water = h00 == 0 && h01 == 0 && h10 == 0;
                let tri_b_water = h11 == 0 && h01 == 0 && h10 == 0;

                let is_walkable = region_map.is_walkable(tile);

                let color_a = if is_building {
                    building_color
                } else if tri_a_water {
                    water_color
                } else if !is_walkable {
                    shore_color
                } else {
                    walkable_color
                };
                let color_b = if is_building {
                    building_color
                } else if tri_b_water {
                    water_color
                } else if !is_walkable {
                    shore_color
                } else {
                    walkable_color
                };

                if is_building { count_building += 1; }
                else if tri_a_water && tri_b_water { count_water += 1; }
                else if !is_walkable { /* shore buffer, counted implicitly */ }
                else { count_walkable += 1; }

                // Corner positions: 0=TL, 1=TR, 2=BR, 3=BL
                let corners: [(f32, f32); 4] = [
                    (cell_x as f32, cell_y as f32),
                    (cell_x as f32 + 1.0, cell_y as f32),
                    (cell_x as f32 + 1.0, cell_y as f32 + 1.0),
                    (cell_x as f32, cell_y as f32 + 1.0),
                ];

                // Emit 6 vertices (2 triangles with independent colors)
                // Triangle A: TL(0), BL(3), TR(1)
                let tri_a_corners = [0usize, 3, 1];
                // Triangle B: BR(2), BL(3), TR(1)
                let tri_b_corners = [2usize, 3, 1];

                for (tri_corners, color) in [(&tri_a_corners, color_a), (&tri_b_corners, color_b)] {
                    for &ci in tri_corners {
                        let (cx, cy) = corners[ci];
                        let vis_x = ((cx - shift.x as f32) % wf + wf) % wf;
                        let vis_y = ((cy - shift.y as f32) % wf + wf) % wf;
                        let gx = vis_x * step;
                        let gy = vis_y * step;

                        let h = landscape.interpolate_height_at(cx, cy);
                        let vdx = gx - center;
                        let vdy = gy - center;
                        let curvature = (vdx * vdx + vdy * vdy) * cs;
                        let gz = h - curvature + height_offset;

                        model.push_vertex(ColorVertex {
                            coord: Vector3::new(gx, gy, gz),
                            color,
                        });
                    }
                }
            }
        }

        log::info!("[walkability] water={} building={} walkable={} total={}", count_water, count_building, count_walkable, w*w);
        if model.vertices.is_empty() {
            self.model_walkability = None;
        } else {
            let m = vec![(RenderType::Triangles, model)];
            self.model_walkability = Some(ModelEnvelop::<ColorModel>::new(&gpu.device, m));
        }
    }

    fn rebuild_hud(&mut self) {
        // HUD is rebuilt each frame in draw_hud(), nothing needed here
    }

    fn draw_hud(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let gpu = match self.gpu.as_ref() {
            Some(g) => g,
            None => return,
        };

        // Build HUD data from game state (decoupled)
        let hud_state = self.engine.build_hud_state();
        let layout = hud::compute_hud_layout(self.engine.screen.width as f32, self.engine.screen.height as f32);

        let hud = match self.hud.as_mut() {
            Some(h) => h,
            None => return,
        };

        // Update minimap texture from data contract
        hud.update_minimap(&gpu.device, &gpu.queue, &hud_state.minimap);

        hud.begin_frame();

        // === Left Sidebar Background ===
        hud.draw_rect(0.0, 0.0, layout.sidebar_w, layout.screen_h, [0.08, 0.08, 0.12, 0.92]);

        // === Minimap border ===
        hud.draw_rect(layout.mm_x - 1.0, layout.mm_y - 1.0, layout.mm_size + 2.0, layout.mm_size + 2.0, [0.3, 0.3, 0.4, 1.0]);
        let minimap_rect = Some((layout.mm_x, layout.mm_y, layout.mm_size, layout.mm_size));

        // === Tab Buttons ===
        let tabs = [("Spells", HudTab::Spells), ("Build", HudTab::Buildings), ("Units", HudTab::Units)];
        for (i, (label, tab_id)) in tabs.iter().enumerate() {
            let tx = layout.mm_pad + i as f32 * layout.tab_w;
            let is_active = hud_state.active_tab == *tab_id;
            let bg = if is_active { [0.25, 0.25, 0.4, 1.0] } else { [0.12, 0.12, 0.18, 1.0] };
            hud.draw_rect(tx, layout.tab_y, layout.tab_w - 1.0, layout.tab_h, bg);
            let text_color = if is_active { [1.0, 1.0, 1.0, 1.0] } else { [0.6, 0.6, 0.6, 1.0] };
            hud.draw_text(label, tx + 3.0, layout.tab_y + 3.0 * layout.scale_y, layout.small_font, text_color);
        }

        // === Panel Content (from HudState data contract) ===
        match hud_state.active_tab {
            HudTab::Spells => {
                for (i, entry) in hud_state.panel_entries.iter().enumerate() {
                    let row = i / 2;
                    let col = i % 2;
                    let sx = layout.mm_pad + col as f32 * (layout.sidebar_w / 2.0 - layout.mm_pad);
                    let sy = layout.panel_y + row as f32 * layout.line_h;
                    hud.draw_text(&entry.label, sx, sy, layout.small_font, entry.color);
                }
            }
            _ => {
                for (i, entry) in hud_state.panel_entries.iter().enumerate() {
                    let sy = layout.panel_y + i as f32 * layout.line_h;
                    hud.draw_text(&entry.label, layout.mm_pad, sy, layout.small_font, entry.color);
                }
            }
        }

        // === Viewport Info ===
        let vp_x = layout.sidebar_w + 8.0;
        hud.draw_text(help_text(), vp_x, 8.0, layout.font_scale, [1.0, 1.0, 1.0, 0.7]);

        let info = format!("Level: {}  Frame: {}", hud_state.level_num, hud_state.frame_count);
        hud.draw_text(&info, vp_x, layout.screen_h - layout.font_scale - 4.0, layout.font_scale, [1.0, 1.0, 0.5, 0.7]);

        // === Tribe population (from HudState data contract) ===
        let pop_x = layout.screen_w - 100.0 * layout.scale_x;
        for tp in &hud_state.tribe_populations {
            let py = 8.0 + tp.tribe_index as f32 * (layout.font_scale + 2.0);
            let text = format!("T{}: {}", tp.tribe_index, tp.count);
            hud.draw_text(&text, pop_x, py, layout.font_scale, tp.color);
        }

        // Render
        hud.render_full(encoder, view, &gpu.queue, layout.screen_w, layout.screen_h, minimap_rect);
    }

    fn draw_compass(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let gpu = match self.gpu.as_ref() {
            Some(g) => g,
            None => return,
        };
        let hud = match self.hud.as_mut() {
            Some(h) => h,
            None => return,
        };

        let sw = self.engine.screen.width as f32;
        let sh = self.engine.screen.height as f32;
        let scale = sw / 640.0;

        let radius = 32.0 * scale;
        let cx = sw - radius - 16.0 * scale;
        let cy = sh - radius - 16.0 * scale;
        let bg_size = radius * 2.0 + 8.0 * scale;

        let yaw_rad = (self.engine.camera.angle_z as f32).to_radians();

        hud.begin_frame();

        // Background
        hud.draw_rect(
            cx - radius - 4.0 * scale,
            cy - radius - 4.0 * scale,
            bg_size,
            bg_size,
            [0.0, 0.0, 0.0, 0.55],
        );

        // Cardinal directions: (label, angle_offset, color)
        // angle_offset is the world bearing: N=0, E=90, S=180, W=270
        let font = 8.0 * scale;
        let cardinals: [(&str, f32, [f32; 4]); 4] = [
            ("N", 0.0,   [1.0, 0.3, 0.3, 1.0]),
            ("E", 90.0,  [1.0, 1.0, 1.0, 0.8]),
            ("S", 180.0, [1.0, 1.0, 1.0, 0.8]),
            ("W", 270.0, [1.0, 1.0, 1.0, 0.8]),
        ];

        for (label, bearing, color) in &cardinals {
            let angle = (bearing - self.engine.camera.angle_z as f32).to_radians();
            // Screen: sin for x, -cos for y (up is negative y)
            let lx = cx + radius * 0.75 * angle.sin() - font * 0.3;
            let ly = cy - radius * 0.75 * angle.cos() - font * 0.4;
            hud.draw_text(label, lx, ly, font, *color);
        }

        // Center dot
        let dot = 3.0 * scale;
        hud.draw_rect(cx - dot, cy - dot, dot * 2.0, dot * 2.0, [1.0, 1.0, 1.0, 0.6]);

        // North indicator line from center toward N
        let n_angle = -yaw_rad;
        let line_len = radius * 0.45;
        let nx = cx + line_len * n_angle.sin();
        let ny = cy - line_len * n_angle.cos();
        let tick = 2.0 * scale;
        hud.draw_rect(nx - tick, ny - tick, tick * 2.0, tick * 2.0, [1.0, 0.3, 0.3, 0.9]);

        hud.render_full(encoder, view, &gpu.queue, sw, sh, None);
    }

    fn rebuild_landscape_variants(&mut self, level_res: &LevelRes) {
        let gpu = self.gpu.as_ref().unwrap();
        let device = &gpu.device;
        let group0_layout = self.landscape_group0_layout.as_ref().unwrap();
        let shadow_group2_layout = self.shadow_recv_group2_layout.as_ref().unwrap();
        let heights_buffer = self.heights_buffer.as_ref().unwrap();
        let watdisp_buffer = self.watdisp_buffer.as_ref().unwrap();

        let vertex_layouts = LandscapeModel::vertex_buffer_layouts();
        let surface_format = gpu.surface_format();

        self.program_container = LandscapeProgramContainer::new();

        // CPU palette index variant
        if self.engine.config.cpu {
            let land_texture = make_texture_land(level_res, None);
            let size = (level_res.landscape.land_size() * 32) as u32;

            let cpu_tex = GpuTexture::new_2d(
                device, &gpu.queue, size, size,
                wgpu::TextureFormat::R8Uint, &land_texture, "cpu_land_texture",
            );

            let palette_packed = pack_palette_rgba(&level_res.params.palette);
            let palette_bytes: &[u8] = bytemuck::cast_slice(&palette_packed);
            let palette_buf = GpuBuffer::new_storage(device, palette_bytes, "palette_buffer");

            let group1_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("landscape_cpu_group1"),
                entries: &[
                    make_storage_entry(0), // heights
                    make_storage_entry(1), // watdisp
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Uint,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    make_storage_entry(3), // palette
                ],
            });

            let bind_group_1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("landscape_cpu_bg1"),
                layout: &group1_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: heights_buffer.buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: watdisp_buffer.buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&cpu_tex.view) },
                    wgpu::BindGroupEntry { binding: 3, resource: palette_buf.buffer.as_entire_binding() },
                ],
            });

            let shader_source = include_str!("../../shaders/landscape_cpu.wgsl");
            let pipeline = create_pipeline(device, shader_source, &vertex_layouts, &[group0_layout, &group1_layout, shadow_group2_layout], surface_format, true, wgpu::PrimitiveTopology::TriangleList, "landscape_cpu");
            self.program_container.add(LandscapeVariant { pipeline, bind_group_1 });
        }

        // CPU full texture variant
        if self.engine.config.cpu_full {
            let land_texture = make_texture_land(level_res, None);
            let size = (level_res.landscape.land_size() * 32) as u32;
            let full_tex_data = draw_texture_u8(&level_res.params.palette, size as usize, &land_texture);

            // draw_texture_u8 returns RGB data; need RGBA for wgpu
            let rgba_data = rgb_to_rgba(&full_tex_data);
            let full_tex = GpuTexture::new_2d(
                device, &gpu.queue, size, size,
                wgpu::TextureFormat::Rgba8UnormSrgb, &rgba_data, "cpu_full_land_texture",
            );
            let sampler = GpuTexture::create_sampler(device, false);

            let group1_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("landscape_full_group1"),
                entries: &[
                    make_storage_entry(0), // heights
                    make_storage_entry(1), // watdisp
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

            let bind_group_1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("landscape_full_bg1"),
                layout: &group1_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: heights_buffer.buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: watdisp_buffer.buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&full_tex.view) },
                    wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&sampler) },
                ],
            });

            let shader_source = include_str!("../../shaders/landscape_full.wgsl");
            let pipeline = create_pipeline(device, shader_source, &vertex_layouts, &[group0_layout, &group1_layout, shadow_group2_layout], surface_format, true, wgpu::PrimitiveTopology::TriangleList, "landscape_full");
            self.program_container.add(LandscapeVariant { pipeline, bind_group_1 });
        }

        // Main GPU landscape
        {
            let palette_packed = pack_palette_rgba(&level_res.params.palette);
            let palette_bytes: &[u8] = bytemuck::cast_slice(&palette_packed);
            let palette_buf = GpuBuffer::new_storage(device, palette_bytes, "main_palette_buffer");

            let disp_vec: Vec<i32> = level_res.params.disp0.iter().map(|v| *v as i32).collect();
            let disp_bytes: &[u8] = bytemuck::cast_slice(&disp_vec);
            let disp_buf = GpuBuffer::new_storage(device, disp_bytes, "disp_buffer");

            let bigf_vec: Vec<u32> = level_res.params.bigf0.iter().map(|v| *v as u32).collect();
            let bigf_bytes: &[u8] = bytemuck::cast_slice(&bigf_vec);
            let bigf_buf = GpuBuffer::new_storage(device, bigf_bytes, "bigf_buffer");

            let sla_vec: Vec<u32> = level_res.params.static_landscape_array.iter().map(|v| *v as u32).collect();
            let sla_bytes: &[u8] = bytemuck::cast_slice(&sla_vec);
            let sla_buf = GpuBuffer::new_storage(device, sla_bytes, "sla_buffer");

            let group1_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("landscape_main_group1"),
                entries: &[
                    make_storage_entry(0), // heights
                    make_storage_entry(1), // watdisp
                    make_storage_entry(2), // palette
                    make_storage_entry(3), // disp
                    make_storage_entry(4), // bigf
                    make_storage_entry(5), // sla
                ],
            });

            let bind_group_1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("landscape_main_bg1"),
                layout: &group1_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: heights_buffer.buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: watdisp_buffer.buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: palette_buf.buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 3, resource: disp_buf.buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 4, resource: bigf_buf.buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 5, resource: sla_buf.buffer.as_entire_binding() },
                ],
            });

            let shader_source = include_str!("../../shaders/landscape.wgsl");
            let pipeline = create_pipeline(device, shader_source, &vertex_layouts, &[group0_layout, &group1_layout, shadow_group2_layout], surface_format, true, wgpu::PrimitiveTopology::TriangleList, "landscape_main");
            self.program_container.add(LandscapeVariant { pipeline, bind_group_1 });
        }

        // Gradient variant
        {
            let group1_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("landscape_grad_group1"),
                entries: &[
                    make_storage_entry(0), // heights
                    make_storage_entry(1), // watdisp
                ],
            });

            let bind_group_1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("landscape_grad_bg1"),
                layout: &group1_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: heights_buffer.buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: watdisp_buffer.buffer.as_entire_binding() },
                ],
            });

            let shader_source = include_str!("../../shaders/landscape_grad.wgsl");
            let pipeline = create_pipeline(device, shader_source, &vertex_layouts, &[group0_layout, &group1_layout, shadow_group2_layout], surface_format, true, wgpu::PrimitiveTopology::TriangleList, "landscape_grad");
            self.program_container.add(LandscapeVariant { pipeline, bind_group_1 });
        }
    }

    fn render(&mut self) {
        let frame = self.engine.frame_state(&self.input.drag_state);
        let gpu = self.gpu.as_ref().unwrap();

        // Update uniforms
        let center = self.engine.world_center();
        let focus = Vector3::new(center, center, 0.0);
        let min_z = self.engine.camera_min_z();
        let mvp = MVP::with_zoom(frame.screen, frame.camera, frame.zoom, focus, min_z);
        let mvp_m = mvp.projection * mvp.view * mvp.transform;
        let mvp_raw: TransformRaw = mvp_m.into();
        self.mvp_buffer.as_ref().unwrap().update(&gpu.queue, 0, bytemuck::bytes_of(&mvp_raw));

        // Update model transform
        if let Some(ref model_main) = self.model_main {
            model_main.write_transform(&gpu.queue, &self.model_transform_buffer.as_ref().unwrap().buffer, 0);
        }

        // Update landscape params
        let params = self.engine.build_landscape_params();
        self.landscape_params_buffer.as_ref().unwrap().update(&gpu.queue, 0, bytemuck::bytes_of(&params));

        // Update selection uniform
        #[repr(C)]
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        struct ObjectParams { num_colors: i32, _pad: i32 }
        let obj_params = ObjectParams { num_colors: obj_colors().len() as i32, _pad: 0 };
        self.select_params_buffer.as_ref().unwrap().update(&gpu.queue, 0, bytemuck::bytes_of(&obj_params));

        // Update sky yaw offset
        if let Some(ref sky_buf) = self.sky_uniform_buffer {
            // angle_z is in degrees; map to 0..1 range for UV offset
            let yaw = (frame.camera.angle_z as f32) / 360.0;
            sky_buf.update(&gpu.queue, 0, bytemuck::bytes_of(&[yaw, 0.0f32, 0.0f32, 0.0f32]));
        }

        // Update lighting params (shared by buildings, sprites, shadows)
        if let Some(ref buf) = self.lighting_buffer {
            let lm_center = (self.engine.landscape_mesh.width() - 1) as f32 * self.engine.landscape_mesh.step() / 2.0;
            let vp_radius = lm_center * 0.9;
            let light_data: [f32; 8] = if frame.show_lighting {
                let lx = frame.sunlight.x;
                let ly = frame.sunlight.y;
                let len = (lx * lx + ly * ly + 200.0 * 200.0_f32).sqrt();
                [-lx / len, -ly / len, 200.0 / len, 0.35,
                 lm_center, lm_center, vp_radius, self.engine.game_world.game_tick as f32]
            } else {
                [0.0, 0.0, 1.0, 1.0,
                 lm_center, lm_center, vp_radius, self.engine.game_world.game_tick as f32]
            };
            buf.update(&gpu.queue, 0, bytemuck::bytes_of(&light_data));
        }

        // Update shadow light MVP
        if let Some(ref buf) = self.light_mvp_buffer {
            let lm_center = (self.engine.landscape_mesh.width() - 1) as f32 * self.engine.landscape_mesh.step() / 2.0;
            let vp_radius = lm_center * 0.9;
            let world_center = lm_center * LANDSCAPE_SCALE + LANDSCAPE_OFFSET;
            let world_radius = vp_radius * LANDSCAPE_SCALE;
            let light_mvp = compute_light_mvp(
                &frame.sunlight,
                Point3::new(world_center, world_center, 0.0),
                world_radius,
            );
            let light_raw: TransformRaw = light_mvp.into();
            buf.update(&gpu.queue, 0, bytemuck::bytes_of(&light_raw));
        }

        // Update select model vertex data
        let output = match gpu.surface.get_current_texture() {
            Ok(t) => t,
            Err(wgpu::SurfaceError::Lost) => return,
            Err(wgpu::SurfaceError::OutOfMemory) => panic!("Out of GPU memory"),
            Err(e) => {
                log::error!("Surface error: {:?}", e);
                return;
            }
        };
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("render_encoder"),
        });

        // Shadow depth pass: render buildings + sprites from light's POV
        if frame.show_shadows {
            if let (Some(ref shadow_view), Some(ref shadow_g0)) =
                (&self.shadow_depth_view, &self.shadow_pass_group0)
            {
                let mut shadow_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("shadow_depth_pass"),
                    color_attachments: &[],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: shadow_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    ..Default::default()
                });

                // Shadow cast: buildings
                if let (Some(ref pipeline), Some(ref bg1)) =
                    (&self.shadow_depth_building_pipeline, &self.building_bind_group_1)
                {
                    shadow_pass.set_pipeline(pipeline);
                    shadow_pass.set_bind_group(0, shadow_g0, &[]);
                    shadow_pass.set_bind_group(1, bg1, &[]);
                    if let Some(ref model) = self.model_buildings {
                        model.draw(&mut shadow_pass);
                    }
                }

            }
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &gpu.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            // Draw sky background
            if let (Some(ref sky_pipe), Some(ref sky_bg)) =
                (&self.sky_pipeline, &self.sky_bind_group)
            {
                render_pass.set_pipeline(sky_pipe);
                render_pass.set_bind_group(0, sky_bg, &[]);
                render_pass.draw(0..3, 0..1);
            }

            // Set shadow map bind group 2 (persists across pipeline switches)
            if let Some(ref shadow_g2) = self.shadow_recv_group2 {
                render_pass.set_bind_group(2, shadow_g2, &[]);
            }

            // Draw landscape
            if let Some(variant) = self.program_container.current() {
                render_pass.set_pipeline(&variant.pipeline);
                render_pass.set_bind_group(0, self.landscape_group0_bind_group.as_ref().unwrap(), &[]);
                render_pass.set_bind_group(1, &variant.bind_group_1, &[]);
                if let Some(ref model_main) = self.model_main {
                    model_main.draw(&mut render_pass);
                }
            }

            // Draw person unit sprites (per-type atlas)
            if let (Some(ref spawn_pipeline), Some(ref bg0)) = (&self.spawn_pipeline, &self.building_bind_group_0) {
                render_pass.set_pipeline(spawn_pipeline);
                render_pass.set_bind_group(0, bg0, &[]);
                for ur in &self.unit_renders {
                    if let Some(ref model) = ur.model {
                        render_pass.set_bind_group(1, &ur.bind_group, &[]);
                        model.draw(&mut render_pass);
                    }
                }
            }

            // Draw 3D building meshes
            if frame.show_objects {
                if let (Some(ref pipeline), Some(ref bg0), Some(ref bg1)) =
                    (&self.building_pipeline, &self.building_bind_group_0, &self.building_bind_group_1)
                {
                    render_pass.set_pipeline(pipeline);
                    render_pass.set_bind_group(0, bg0, &[]);
                    render_pass.set_bind_group(1, bg1, &[]);
                    if let Some(ref model) = self.model_buildings {
                        model.draw(&mut render_pass);
                    }

                    // Ghost preview rendering — transparent building at placement position.
                    // When ghost_preview is Some, we would draw the building mesh a second time
                    // at the ghost position with alpha blending. The ghost_alpha and ghost_tint
                    // uniforms control transparency and color:
                    //   - ghost_alpha = 0.5 (semi-transparent)
                    //   - ghost_tint = [0.3, 1.0, 0.3] if valid (green)
                    //   - ghost_tint = [1.0, 0.3, 0.3] if invalid (red)
                    // NOTE: Full GPU uniform buffer + shader integration deferred to render
                    // pipeline refactor. This placeholder documents the rendering intent.
                    if let Some(ref ghost) = frame.ghost_preview {
                        let _ghost_alpha: f32 = 0.5;
                        let _ghost_tint: [f32; 3] = if ghost.valid {
                            [0.3, 1.0, 0.3] // green = valid placement
                        } else {
                            [1.0, 0.3, 0.3] // red = invalid placement
                        };
                        // TODO: Create separate ghost uniform buffer, bind ghost_alpha/ghost_tint
                        // to building fragment shader, draw building mesh at ghost position with
                        // BlendState::ALPHA_BLENDING enabled on the building pipeline color state.
                        log::trace!(
                            "[ghost] preview type={} at ({},{}) valid={}",
                            ghost.building_type, ghost.cell_x, ghost.cell_y, ghost.valid
                        );
                    }
                }
            }

            // Draw level object markers (non-building objects)
            if frame.show_objects {
                if let Some(ref marker_pipeline) = self.objects_marker_pipeline {
                    render_pass.set_pipeline(marker_pipeline);
                    render_pass.set_bind_group(0, self.objects_group0_bind_group.as_ref().unwrap(), &[]);
                    if let Some(ref model_objects) = self.model_objects {
                        model_objects.draw(&mut render_pass);
                    }
                }
            }

            // Draw selection outlines (always) and unit marker billboards (toggled)
            if let Some(ref marker_pipeline) = self.objects_marker_pipeline {
                render_pass.set_pipeline(marker_pipeline);
                render_pass.set_bind_group(0, self.objects_group0_bind_group.as_ref().unwrap(), &[]);
                if frame.show_markers {
                    if let Some(ref model) = self.model_unit_markers {
                        model.draw(&mut render_pass);
                    }
                }
                if let Some(ref model) = self.model_selection_outlines {
                    model.draw(&mut render_pass);
                }
            }

            // Draw walkability debug overlay (F8 toggle)
            if self.engine.walkability_visible {
                if let Some(ref walk_pipeline) = self.walkability_pipeline {
                    render_pass.set_pipeline(walk_pipeline);
                    render_pass.set_bind_group(0, self.objects_group0_bind_group.as_ref().unwrap(), &[]);
                    if let Some(ref model) = self.model_walkability {
                        model.draw(&mut render_pass);
                    }
                }
            }

        }

        // HUD pass (2D overlay — no depth, load existing color)
        // End borrows before calling draw_hud (needs &mut self)
        let _ = gpu;
        drop(frame);
        if self.engine.hud_visible {
            self.draw_hud(&mut encoder, &view);
        }
        if self.engine.compass_visible {
            self.draw_compass(&mut encoder, &view);
        }

        let gpu = self.gpu.as_ref().unwrap();
        gpu.queue.submit(std::iter::once(encoder.finish()));

        if let Some(path) = self.screenshot_path.take() {
            self.capture_screenshot(&output.texture, &path);
        }

        output.present();
    }

    fn capture_screenshot(&self, texture: &wgpu::Texture, path: &str) {
        let gpu = match self.gpu.as_ref() {
            Some(g) => g,
            None => return,
        };
        let width = gpu.config.width;
        let height = gpu.config.height;
        let bytes_per_pixel = 4u32;
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let padded_bytes_per_row = (unpadded_bytes_per_row + 255) & !255;
        let buffer_size = (padded_bytes_per_row * height) as u64;

        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            size: buffer_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
            label: Some("screenshot_buffer"),
        });

        let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("screenshot_encoder"),
        });
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        );
        gpu.queue.submit(Some(encoder.finish()));

        let slice = buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| { let _ = tx.send(r); });
        let _ = gpu.device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None });

        match rx.recv() {
            Ok(Ok(())) => {
                let data = slice.get_mapped_range();
                let mut pixels = Vec::with_capacity((width * height * 4) as usize);
                for row in 0..height {
                    let offset = (row * padded_bytes_per_row) as usize;
                    let row_data = &data[offset..offset + (width * 4) as usize];
                    for pixel in row_data.chunks(4) {
                        // BGRA → RGBA (force alpha=255, surface alpha is unreliable on macOS)
                        pixels.push(pixel[2]); // R
                        pixels.push(pixel[1]); // G
                        pixels.push(pixel[0]); // B
                        pixels.push(255);      // A
                    }
                }
                drop(data);
                buffer.unmap();

                if let Some(img) = image::RgbaImage::from_raw(width, height, pixels) {
                    match img.save(path) {
                        Ok(()) => eprintln!("[screenshot] saved {}", path),
                        Err(e) => eprintln!("[screenshot] save error: {}", e),
                    }
                }
            }
            _ => {
                eprintln!("[screenshot] buffer map failed");
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::default().with_title("Faithful"))
                .unwrap(),
        );
        self.window = Some(window.clone());

        let gpu = pollster::block_on(GpuContext::new(window));
        let device = &gpu.device;

        let base = self.engine.config.base.clone().unwrap_or_else(|| Path::new("/opt/sandbox/pop").to_path_buf());
        let level_type = self.engine.config.landtype.as_deref();
        let level_res = LevelRes::new(&base, self.engine.level_num, level_type);

        let shores = level_res.landscape.make_shores();
        self.engine.landscape_mesh.set_heights(&shores.height);

        // Heights storage buffer
        let heights_vec = shores.to_vec();
        let heights_bytes: &[u8] = bytemuck::cast_slice(&heights_vec);
        let heights_buffer = GpuBuffer::new_storage(device, heights_bytes, "heights_buffer");

        // Watdisp storage buffer
        let watdisp_vec: Vec<u32> = level_res.params.watdisp.iter().map(|v| *v as u32).collect();
        let watdisp_bytes: &[u8] = bytemuck::cast_slice(&watdisp_vec);
        let watdisp_buffer = GpuBuffer::new_storage(device, watdisp_bytes, "watdisp_buffer");

        // Shared uniform buffers
        let mvp_buffer = GpuBuffer::new_uniform(device, 64, "mvp_buffer");
        let model_transform_buffer = GpuBuffer::new_uniform(device, 64, "model_transform_buffer");
        let landscape_params_buffer = GpuBuffer::new_uniform_init(
            device,
            bytemuck::bytes_of(&self.engine.build_landscape_params()),
            "landscape_params_buffer",
        );

        // Landscape group 0 layout and bind group
        let landscape_group0_layout = create_landscape_group0_layout(device);
        let landscape_group0_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("landscape_group0_bg"),
            layout: &landscape_group0_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: mvp_buffer.buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: model_transform_buffer.buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: landscape_params_buffer.buffer.as_entire_binding() },
            ],
        });

        // Objects (selection lines) setup
        let objects_group0_layout = create_objects_group0_layout(device);
        let objects_group0_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("objects_group0_bg"),
            layout: &objects_group0_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: mvp_buffer.buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: model_transform_buffer.buffer.as_entire_binding() },
            ],
        });

        // Objects group 1: params uniform + color storage
        let objects_group1_layout = create_objects_group1_layout(device);

        #[repr(C)]
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        struct ObjectParams { num_colors: i32, _pad: i32 }

        let colors = obj_colors();
        let obj_params = ObjectParams { num_colors: colors.len() as i32, _pad: 0 };
        let select_params_buffer = GpuBuffer::new_uniform_init(device, bytemuck::bytes_of(&obj_params), "select_params_buffer");

        // Pack colors as vec4<u32> (RGBA, each channel widened to u32)
        let color_data: Vec<[u32; 4]> = colors.iter().map(|c| {
            [c.x as u32, c.y as u32, c.z as u32, 0u32]
        }).collect();
        let color_bytes: &[u8] = bytemuck::cast_slice(&color_data);
        let color_buffer = GpuBuffer::new_storage(device, color_bytes, "obj_color_buffer");

        let objects_group1_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("objects_group1_bg"),
            layout: &objects_group1_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: select_params_buffer.buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: color_buffer.buffer.as_entire_binding() },
            ],
        });

        // Landscape model
        let model_main = make_landscape_model(device, &self.engine.landscape_mesh);

        // Shaman sprite atlas and pipeline
        let sprite_group1_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite_group1_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Lit group 0 layout: MVP + model_transform + lighting (shared by sprites, buildings)
        let lit_group0_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("lit_group0_layout"),
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
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Shadow receive group 2 layout (shared by all shadow-receiving shaders)
        let shadow_recv_group2_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shadow_recv_group2_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Shadow mapping resources
        let shadow_map_size = 2048u32;
        let shadow_depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow_depth_texture"),
            size: wgpu::Extent3d { width: shadow_map_size, height: shadow_map_size, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let shadow_depth_view = shadow_depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let shadow_comparison_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow_comparison_sampler"),
            compare: Some(wgpu::CompareFunction::LessEqual),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let light_mvp_buffer = GpuBuffer::new_uniform_init(
            device, bytemuck::bytes_of(&TransformRaw::from(Matrix4::<f32>::identity())), "light_mvp_buffer",
        );

        // Shadow depth pass group 0: light_mvp + model_transform
        let shadow_pass_group0_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shadow_pass_group0_layout"),
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
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let shadow_pass_group0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow_pass_group0"),
            layout: &shadow_pass_group0_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: light_mvp_buffer.buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: model_transform_buffer.buffer.as_entire_binding() },
            ],
        });

        // Shadow receive group 2 bind group
        let shadow_recv_group2 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow_recv_group2"),
            layout: &shadow_recv_group2_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&shadow_depth_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&shadow_comparison_sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: light_mvp_buffer.buffer.as_entire_binding() },
            ],
        });

        // Lighting uniform buffer (sun_dir + ambient + camera_focus + viewport_radius + game_tick)
        let center = (self.engine.landscape_mesh.width() - 1) as f32 * self.engine.landscape_mesh.step() / 2.0;
        let vp_radius = center * 0.9;
        let lx = self.engine.sunlight.x;
        let ly = self.engine.sunlight.y;
        let len = (lx * lx + ly * ly + 200.0 * 200.0_f32).sqrt();
        let light_data: [f32; 8] = [
            -lx / len, -ly / len, 200.0 / len, 0.35,
            center, center, vp_radius, 0.0,
        ];
        let lighting_buffer = GpuBuffer::new_uniform_init(
            device, bytemuck::bytes_of(&light_data), "lighting_buffer",
        );

        let lit_group0_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("lit_group0_bg"),
            layout: &lit_group0_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: mvp_buffer.buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: model_transform_buffer.buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: lighting_buffer.buffer.as_entire_binding() },
            ],
        });

        // Shadow depth building pipeline (depth-only, no color target)
        let shadow_depth_building_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shadow_depth_building_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/shadow_depth_building.wgsl").into()),
        });
        let shadow_depth_building_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("shadow_depth_building_layout"),
            bind_group_layouts: &[&shadow_pass_group0_layout, &sprite_group1_layout],
            immediate_size: 0,
        });
        let shadow_depth_building_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shadow_depth_building_pipeline"),
            layout: Some(&shadow_depth_building_layout),
            vertex: wgpu::VertexState {
                module: &shadow_depth_building_shader,
                entry_point: Some("vs_main"),
                buffers: &TexModel::vertex_buffer_layouts(),
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shadow_depth_building_shader,
                entry_point: Some("fs_main"),
                targets: &[],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // Shaman sprite pipeline (with lighting + shadow receiving)
        let spawn_shader_source = include_str!("../../shaders/shaman_sprite.wgsl");
        let spawn_vertex_layouts = TexModel::vertex_buffer_layouts();
        let spawn_pipeline = create_pipeline(
            device, spawn_shader_source, &spawn_vertex_layouts,
            &[&lit_group0_layout, &sprite_group1_layout, &shadow_recv_group2_layout],
            gpu.surface_format(), true,
            wgpu::PrimitiveTopology::TriangleList,
            "shaman_sprite_pipeline",
        );

        // Level objects marker pipeline (group 0 only, no group 1)
        let objects_marker_shader = include_str!("../../shaders/level_objects.wgsl");
        let objects_marker_layouts = ColorModel::vertex_buffer_layouts();
        let objects_marker_pipeline = create_pipeline(
            device, objects_marker_shader, &objects_marker_layouts,
            &[&objects_group0_layout],
            gpu.surface_format(), true,
            wgpu::PrimitiveTopology::TriangleList,
            "level_objects_pipeline",
        );

        // Walkability overlay pipeline (alpha blended)
        let walkability_shader = include_str!("../../shaders/walkability_overlay.wgsl");
        let walkability_layouts = ColorModel::vertex_buffer_layouts();
        let walkability_pipeline = create_pipeline_blended(
            device, walkability_shader, &walkability_layouts,
            &[&objects_group0_layout],
            gpu.surface_format(), true,
            wgpu::PrimitiveTopology::TriangleList,
            "walkability_overlay_pipeline",
            wgpu::BlendState::ALPHA_BLENDING,
        );

        // Unit sprite atlases are built after self.gpu is set (see below)

        // Load dual OBJS banks: bank 0 for buildings, level bank for scenery.
        // Bank 0 has building models at indices 117-193 (building_obj_index).
        // Level banks have scenery at different indices (scenery_obj_index).
        // Shape_LoadBank @ 0x49b990 remaps bank 0 → 2.
        let (building_objects, scenery_objects) = Object3D::load_dual_banks(&base, level_res.obj_bank);
        let level_bank = if level_res.obj_bank == 0 { 2 } else { level_res.obj_bank };
        let bld_count = building_objects.iter().filter(|o| o.is_some()).count();
        let scn_count = scenery_objects.iter().filter(|o| o.is_some()).count();
        eprintln!("[OBJS] buildings: bank=0 entries={} non-empty={}",
            building_objects.len(), bld_count);
        eprintln!("[OBJS] scenery:   bank={} entries={} non-empty={}",
            level_bank, scenery_objects.len(), scn_count);
        let bank_str = level_bank.to_string();
        let obj_paths = ObjectPaths::from_default_dir(&base, &bank_str);
        let shape_footprints = ShapeFootprints::from_file(&obj_paths.shapes);
        let shapes: Vec<Shape> = shape_footprints.shapes().to_vec();
        eprintln!("[shapes] loaded {} entries with footprint bitmaps", shapes.len());
        for (i, s) in shapes.iter().take(10).enumerate() {
            let sref = s.shape_ref;
            eprintln!("[shapes] [{}] {}x{} origin=({},{}) ref={}",
                i, s.width, s.height, s.origin_x, s.origin_z, sref);
        }

        let (bl320_w, bl320_h, mut bl320_data) = make_bl320_texture_rgba(
            &level_res.paths.bl320, &level_res.params.palette);

        // Mark transparent pixels (palette index 0) with alpha=255 so the shader
        // can discard them via `if (color.w > 0.0) { discard; }`.
        let key_r = level_res.params.palette[0];
        let key_g = level_res.params.palette[1];
        let key_b = level_res.params.palette[2];
        for pixel in bl320_data.chunks_exact_mut(4) {
            if pixel[0] == key_r && pixel[1] == key_g && pixel[2] == key_b && pixel[3] == 0 {
                pixel[3] = 255;
            }
        }

        let bl320_gpu_tex = GpuTexture::new_2d(
            device, &gpu.queue,
            bl320_w as u32, bl320_h as u32,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            &bl320_data,
            "bl320_texture",
        );
        let bl320_sampler = GpuTexture::create_sampler(device, false);

        let building_bind_group_1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("building_bg1"),
            layout: &sprite_group1_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&bl320_gpu_tex.view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&bl320_sampler) },
            ],
        });

        // Building pipeline (objects_tex.wgsl with directional lighting)
        let building_shader_source = include_str!("../../shaders/objects_tex.wgsl");
        let building_vertex_layouts = TexModel::vertex_buffer_layouts();
        let building_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("building_shader"),
            source: wgpu::ShaderSource::Wgsl(building_shader_source.into()),
        });
        let building_pipe_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("building_pipeline_layout"),
            bind_group_layouts: &[&lit_group0_layout, &sprite_group1_layout, &shadow_recv_group2_layout],
            immediate_size: 0,
        });
        let building_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("building_pipeline"),
            layout: Some(&building_pipe_layout),
            vertex: wgpu::VertexState {
                module: &building_shader,
                entry_point: Some("vs_main"),
                buffers: &building_vertex_layouts,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &building_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: gpu.surface_format(),
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        // Sky texture and pipeline
        let sky_data = std::fs::read(&level_res.paths.sky).ok();
        let (sky_pipeline, sky_bind_group, sky_uniform_buffer) = if let Some(sky_raw) = sky_data {
            // sky0-{key}.dat is 512x512 palette indices (262144 bytes).
            // sky0-0.dat is 307200 bytes (600x512); just take first 512 rows.
            let sky_size = 512usize;
            let pixel_count = sky_size * sky_size;
            let sky_indices = if sky_raw.len() >= pixel_count {
                &sky_raw[..pixel_count]
            } else {
                &sky_raw[..]
            };
            let pal = &level_res.params.palette;
            // Game adds 0x70 to every sky byte, then uses result as direct palette index
            let mut sky_rgb = vec![0u8; sky_size * sky_size * 3];
            for (i, &idx) in sky_indices.iter().enumerate() {
                let pal_idx = idx.wrapping_add(0x70) as usize * 4;
                sky_rgb[i * 3]     = pal[pal_idx];
                sky_rgb[i * 3 + 1] = pal[pal_idx + 1];
                sky_rgb[i * 3 + 2] = pal[pal_idx + 2];
            }
            let sky_rgba = rgb_to_rgba(&sky_rgb);
            let sky_tex = GpuTexture::new_2d(
                device, &gpu.queue,
                sky_size as u32, sky_size as u32,
                wgpu::TextureFormat::Rgba8UnormSrgb,
                &sky_rgba, "sky_texture",
            );
            let sky_sampler = GpuTexture::create_sampler(device, false);
            let sky_uniform = GpuBuffer::new_uniform_init(
                device, bytemuck::bytes_of(&[0.0f32; 4]), "sky_uniform",
            );

            let sky_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("sky_bg_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
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

            let sky_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("sky_bg"),
                layout: &sky_bg_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: sky_uniform.buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&sky_tex.view) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&sky_sampler) },
                ],
            });

            let sky_shader_source = include_str!("../../shaders/sky.wgsl");
            let sky_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("sky_shader"),
                source: wgpu::ShaderSource::Wgsl(sky_shader_source.into()),
            });
            let sky_pipe_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("sky_pipeline_layout"),
                bind_group_layouts: &[&sky_bg_layout],
                immediate_size: 0,
            });
            let sky_pipe = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("sky_pipeline"),
                layout: Some(&sky_pipe_layout),
                vertex: wgpu::VertexState {
                    module: &sky_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &sky_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: gpu.surface_format(),
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

            (Some(sky_pipe), Some(sky_bg), Some(sky_uniform))
        } else {
            log::warn!("Sky texture not found: {:?}", level_res.paths.sky);
            (None, None, None)
        };

        // HUD renderer (replaces old overlay text pipeline)
        let hud_renderer = HudRenderer::new(
            device, &gpu.queue, gpu.surface_format(),
            self.engine.screen.width as f32, self.engine.screen.height as f32,
        );

        // Store everything
        self.heights_buffer = Some(heights_buffer);
        self.watdisp_buffer = Some(watdisp_buffer);
        self.mvp_buffer = Some(mvp_buffer);
        self.model_transform_buffer = Some(model_transform_buffer);
        self.landscape_params_buffer = Some(landscape_params_buffer);
        self.select_params_buffer = Some(select_params_buffer);
        self.landscape_group0_layout = Some(landscape_group0_layout);
        self.landscape_group0_bind_group = Some(landscape_group0_bind_group);
        self.objects_group0_bind_group = Some(objects_group0_bind_group);
        self.objects_group1_bind_group = Some(objects_group1_bind_group);
        self.spawn_pipeline = Some(spawn_pipeline);
        self.sprite_group1_layout = Some(sprite_group1_layout);
        self.shadow_depth_view = Some(shadow_depth_view);
        self.shadow_depth_building_pipeline = Some(shadow_depth_building_pipeline);

        self.shadow_pass_group0 = Some(shadow_pass_group0);
        self.light_mvp_buffer = Some(light_mvp_buffer);
        self.shadow_recv_group2_layout = Some(shadow_recv_group2_layout);
        self.shadow_recv_group2 = Some(shadow_recv_group2);
        self.building_bind_group_0 = Some(lit_group0_bind_group);
        self.lighting_buffer = Some(lighting_buffer);
        self.objects_marker_pipeline = Some(objects_marker_pipeline);
        self.walkability_pipeline = Some(walkability_pipeline);
        self.engine.building_objects = building_objects;
        self.engine.scenery_objects = scenery_objects;
        self.engine.shapes = shapes;
        self.engine.shape_footprints = shape_footprints;
        self.building_pipeline = Some(building_pipeline);
        self.building_bind_group_1 = Some(building_bind_group_1);
        self.sky_pipeline = sky_pipeline;
        self.sky_bind_group = sky_bind_group;
        self.sky_uniform_buffer = sky_uniform_buffer;
        self.model_main = Some(model_main);
        self.hud = Some(hud_renderer);

        self.gpu = Some(gpu);

        // Build landscape variants (needs self.gpu, heights_buffer, etc.)
        let base2 = self.engine.config.base.clone().unwrap_or_else(|| Path::new("/opt/sandbox/pop").to_path_buf());
        let level_type2 = self.engine.config.landtype.as_deref();
        let level_res2 = LevelRes::new(&base2, self.engine.level_num, level_type2);
        self.rebuild_landscape_variants(&level_res2);

        // Build per-unit-type sprite atlases
        self.rebuild_unit_atlases(&base2, &level_res2.params.palette);

        self.engine.level_objects = extract_level_objects(&level_res2);

        // Extract person units into the coordinator (they become live entities)
        let shores2 = level_res2.landscape.make_shores();
        self.engine.unit_coordinator.load_level(&level_res2.units, &shores2.height, level_res2.landscape.land_size());
        self.engine.level_objects.retain(|obj| obj.model_type != ModelType::Person);

        // Populate unit_renders cells from live coordinator units
        self.sync_unit_render_cells();

        // Flatten terrain under buildings (modifies heightmap + re-uploads GPU buffer)
        self.flatten_terrain_under_buildings();

        // Mark building footprints in region map for pathfinding walkability
        self.populate_buildings_in_region_map();

        self.rebuild_spawn_model();
        self.center_on_tribe0_shaman();

        // Build HUD sprite atlas from plspanel.spr
        {
            let panel_path = base2.join("data").join("plspanel.spr");
            if let Some(panel_container) = ContainerPSFB::from_file(&panel_path) {
                self.engine.hud_panel_sprite_count = panel_container.len();
                if let Some(ref mut hud) = self.hud {
                    let gpu = self.gpu.as_ref().unwrap();
                    hud.build_atlas(&gpu.device, &gpu.queue, &panel_container, &level_res2.params.palette);
                }
                log::info!("[hud] Loaded {} sprites from plspanel.spr", self.engine.hud_panel_sprite_count);
            } else {
                log::warn!("[hud] plspanel.spr not found at {:?}, using font-only atlas", panel_path);
            }
        }

        self.do_render = true;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: winit::window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::Resized(physical_size) => {
                self.engine.screen.width = physical_size.width;
                self.engine.screen.height = physical_size.height;
                if let Some(gpu) = self.gpu.as_mut() {
                    gpu.resize(physical_size);
                }
                self.rebuild_hud();
                self.do_render = true;
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.input.mouse_pos = Point2::<f32>::new(position.x as f32, position.y as f32);

                // Update drag state
                match self.input.drag_state {
                    DragState::PendingDrag { start } => {
                        let dx = self.input.mouse_pos.x - start.x;
                        let dy = self.input.mouse_pos.y - start.y;
                        if dx * dx + dy * dy > 25.0 { // 5px threshold
                            self.input.drag_state = DragState::Dragging { start, current: self.input.mouse_pos };
                            self.do_render = true;
                        }
                    }
                    DragState::Dragging { start, .. } => {
                        self.input.drag_state = DragState::Dragging { start, current: self.input.mouse_pos };
                        self.do_render = true;
                    }
                    DragState::None => {}
                }
            },
            WindowEvent::MouseInput { state, button, .. } => {
                let layout = hud::compute_hud_layout(self.engine.screen.width as f32, self.engine.screen.height as f32);
                let on_sidebar = self.input.mouse_pos.x < layout.sidebar_w;

                match (button, state) {
                    (MouseButton::Left, ElementState::Pressed) => {
                        if on_sidebar {
                            if let Some(tab) = hud::detect_tab_click(self.input.mouse_pos.x, self.input.mouse_pos.y, &layout) {
                                self.engine.apply_command(&GameCommand::SetHudTab(tab));
                            }
                            self.do_render = true;
                        } else {
                            // Start potential drag (game world interaction)
                            self.input.drag_state = DragState::PendingDrag { start: self.input.mouse_pos };
                        }
                    }
                    (MouseButton::Left, ElementState::Released) => {
                        if !on_sidebar {
                            match self.input.drag_state {
                                DragState::PendingDrag { .. } => {
                                    // Short click (no drag) — resolve screen pos to unit command
                                    let cmd = match self.engine.find_unit_at_screen_pos(&self.input.mouse_pos) {
                                        Some(id) => GameCommand::SelectUnit(id),
                                        None => GameCommand::ClearSelection,
                                    };
                                    self.engine.apply_command(&cmd);
                                }
                                DragState::Dragging { start, current } => {
                                    // Drag release — resolve screen rect to unit IDs
                                    let ids = self.engine.units_in_screen_rect(start, current);
                                    self.engine.apply_command(&GameCommand::SelectMultiple(ids));
                                }
                                DragState::None => {}
                            }
                        }
                        self.input.drag_state = DragState::None;
                        self.rebuild_unit_models();
                    }
                    (MouseButton::Right, ElementState::Pressed) => {
                        if !on_sidebar {
                            // Right-click: resolve screen pos to world coords, then issue move
                            if let Some((cx, cy)) = self.engine.screen_to_cell(&self.input.mouse_pos) {
                                let target = cell_to_world(cx, cy, self.engine.landscape_mesh.width() as f32);
                                let selected = self.engine.unit_coordinator.selection.selected.len();
                                log::info!("[move-order] click cell=({:.1}, {:.1}) → world=({}, {}) selected={}",
                                    cx, cy, target.x, target.z, selected);
                                if selected > 0 {
                                    let uid = self.engine.unit_coordinator.selection.selected[0];
                                    if let Some(u) = self.engine.unit_coordinator.units().get(uid) {
                                        let walkable = self.engine.unit_coordinator.region_map()
                                            .is_walkable(target.to_tile());
                                        log::info!("[move-order] unit {} at world=({}, {}) cell=({:.1}, {:.1}) target_walkable={}",
                                            uid, u.movement.position.x, u.movement.position.z,
                                            u.cell_x, u.cell_y, walkable);
                                    }
                                }
                                self.engine.apply_command(&GameCommand::OrderMove {
                                    x: target.x as f32, z: target.z as f32,
                                });
                            } else {
                                log::warn!("[move-order] screen_to_cell returned None");
                            }
                        }
                    }
                    _ => {}
                }
                self.do_render = true;
            },
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 50.0,
                };
                let new_zoom = self.engine.zoom * 1.1_f32.powf(scroll_y);
                self.engine.apply_command(&GameCommand::SetZoom(new_zoom));
                self.log_camera_state("zoom");
                self.do_render = true;
            },
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    if let PhysicalKey::Code(key) = event.physical_key {
                        if let Some(cmd) = translate_key(key) {
                            let prev_shift = self.engine.landscape_mesh.get_shift_vector();
                            self.engine.apply_command(&cmd);

                            // App-level side effects that need GPU state
                            match &cmd {
                                GameCommand::Quit => { event_loop.exit(); return; }
                                GameCommand::NextShader => { self.program_container.next(); }
                                GameCommand::PrevShader => { self.program_container.prev(); }
                                GameCommand::NextLevel | GameCommand::PrevLevel => {
                                    self.update_level();
                                }
                                GameCommand::CenterOnShaman => {
                                    self.center_on_tribe0_shaman();
                                    self.log_camera_state("space_center");
                                }
                                GameCommand::ResetCamera => {
                                    self.rebuild_spawn_model();
                                    self.log_camera_state("reset");
                                }
                                GameCommand::ToggleCurvature | GameCommand::AdjustCurvature { .. }
                | GameCommand::AdjustSpriteOffset { .. } | GameCommand::AdjustSpriteScale { .. } => {
                                    self.rebuild_spawn_model();
                                }
                                GameCommand::PanScreen { .. } | GameCommand::PanTerrain { .. } => {
                                    self.shaman_pan = None;
                                    let new_shift = self.engine.landscape_mesh.get_shift_vector();
                                    if new_shift != prev_shift {
                                        self.rebuild_spawn_model();
                                        self.log_camera_state(&format!("{:?}", key));
                                    }
                                }
                                GameCommand::RotateCamera { .. } | GameCommand::TiltCamera { .. } => {
                                    self.rebuild_spawn_model();
                                    self.log_camera_state(&format!("{:?}", key));
                                }
                                _ => {}
                            }
                            self.do_render = true;
                        }
                    }
                }
            },
            WindowEvent::RedrawRequested => {
                // Tick game simulation via GameWorld tick loop.
                // UnitCoordinator implements ObjectTick and is plugged into the
                // objects slot, so person state machines run inside the proper
                // tick order (after terrain, before water).
                {
                    let (mut a, mut c, mut d, mut e, mut f, mut g, mut h, mut i, mut j, mut k) =
                        (NoOp, NoOp, NoOp, NoOp, NoOp, NoOp, NoOp, NoOp, NoOp, NoOp);
                    let mut subs = TickSubsystems {
                        terrain: &mut a, objects: &mut self.engine.unit_coordinator,
                        water: &mut c,
                        network: &mut d, actions: &mut e, game_time: &mut f,
                        single_player: &mut g, tutorial: &mut h, ai: &mut i,
                        population: &mut j, mana: &mut k,
                    };
                    let ticks = self.engine.game_world.simulation_tick(&self.engine.game_time, &mut subs);
                    if ticks > 0 {
                        // 7d. Mana generation — runs after object ticks complete.
                        // Uses pool (read-only) from coordinator + tribe data (mutable).
                        // Iterates persons calling mana_rate_for_person + add_mana per tick,
                        // plus housing mana from active huts.
                        {
                            let mut mana_bridge = ManaTickBridge {
                                pool: self.engine.unit_coordinator.pool(),
                                tribes: &mut self.engine.game_world.tribes,
                            };
                            use crate::engine::state::traits::ManaTick;
                            for _ in 0..ticks {
                                mana_bridge.tick_update_mana();
                            }
                        }
                        self.sync_unit_render_cells();
                        self.rebuild_spawn_model();
                        self.rebuild_unit_models();
                        self.do_render = true;
                    }
                }

                // Smooth camera pan to shaman
                self.tick_shaman_pan();

                // Auto-animate water
                self.engine.frame_count = self.engine.frame_count.wrapping_add(1);
                if self.engine.frame_count % self.engine.wat_interval == 0 {
                    self.engine.wat_offset += 1;
                    self.do_render = true;
                }
                if self.do_render && self.gpu.is_some() {
                    self.render();
                    self.do_render = false;
                }
                // Script replay: process one command per frame
                if self.is_script_mode() {
                    if !self.run_script_step() {
                        event_loop.exit();
                        return;
                    }
                }
            },
            _ => (),
        }
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl App {
    pub fn run(config: AppConfig) {
        let event_loop = EventLoop::new().unwrap();
        let mut app = App::new(config);
        event_loop.run_app(&mut app).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toroidal_delta_wrap_forward() {
        assert_eq!(toroidal_delta(120, 5, 128), 13);
    }

    #[test]
    fn toroidal_delta_wrap_backward() {
        assert_eq!(toroidal_delta(5, 120, 128), -13);
    }

    #[test]
    fn toroidal_delta_direct() {
        assert_eq!(toroidal_delta(10, 50, 128), 40);
        assert_eq!(toroidal_delta(50, 10, 128), -40);
    }

    #[test]
    fn toroidal_delta_zero() {
        assert_eq!(toroidal_delta(42, 42, 128), 0);
    }

    #[test]
    fn toroidal_delta_half() {
        // Exactly half — prefer forward
        assert_eq!(toroidal_delta(0, 64, 128), 64);
    }
}
