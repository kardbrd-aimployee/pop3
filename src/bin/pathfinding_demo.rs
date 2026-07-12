//! Pathfinding Demo — Dual-arm wall-following visualizer
//!
//! Two modes:
//!   Test mode (default) — synthetic map with obstacles and preset test cases
//!   Level mode — loads game level data for interactive testing
//!
//! Controls:
//!   Left-click    - Set start position
//!   Right-click   - Set goal position (triggers pathfinding)
//!   M             - Toggle between test mode and level mode
//!   1-9, 0        - Select test case (test mode only)
//!   G             - Toggle cell grid overlay
//!   R             - Reset (clear path and markers)
//!   +/-           - Zoom in/out
//!   Arrow keys    - Pan camera
//!   N/P           - Next/previous level (level mode only)
//!   Escape        - Quit

use std::path::PathBuf;
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes};

use clap::{Arg, Command};

use pop3::data::level::{Landscape, LevelRes};
use pop3::engine::movement::constants::{CELL_HAS_BUILDING, REGION_GRID_SIZE};
use pop3::engine::movement::{self, PathfindDebug, PathfindResult, RegionMap, TileCoord, Waypoint};
use pop3::render::gpu::buffer::GpuBuffer;
use pop3::render::gpu::context::GpuContext;
use pop3::render::gpu::texture::GpuTexture;

const MAP_SIZE: usize = 128;
/// World units per cell
const CELL_SIZE: f32 = 256.0;
/// Full world size in world units
const WORLD_SIZE: f32 = MAP_SIZE as f32 * CELL_SIZE;

/// Max overlay vertices for cell fills + grid lines + path lines
const MAX_OVERLAY_VERTS: usize = 200_000;

// ──────────────────────────────────────────────────────────────────────────────
// Vertex types
// ──────────────────────────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct TerrainVertex {
    position: [f32; 2],
    uv: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct OverlayVertex {
    position: [f32; 2],
    color: [f32; 4],
}

// ──────────────────────────────────────────────────────────────────────────────
// Map modes and test cases
// ──────────────────────────────────────────────────────────────────────────────

enum MapMode {
    Test,
    Level(u8),
}

struct TestCase {
    name: &'static str,
    start: (usize, usize),
    goal: (usize, usize),
    expect_found: bool,
    /// Expected direction(s) that must appear in the path (comma-separated)
    expect_dirs: &'static str,
    /// (min, max) inclusive waypoint count
    expect_wps: (usize, usize),
    /// Why this path shape is expected — for human/AI verification
    rationale: &'static str,
}

const TEST_CASES: &[TestCase] = &[
    TestCase {
        name: "Beeline straight",
        start: (10, 20),
        goal: (35, 20),
        expect_found: true,
        expect_dirs: "E",
        expect_wps: (2, 3),
        rationale: "Straight east, no obstacles between start and goal",
    },
    TestCase {
        name: "Beeline diagonal",
        start: (10, 35),
        goal: (30, 45),
        expect_found: true,
        expect_dirs: "SE",
        expect_wps: (2, 3),
        rationale: "Diagonal SE, no obstacles — LOS optimizer reduces to 2 waypoints",
    },
    TestCase {
        name: "Around block",
        start: (16, 51),
        goal: (28, 51),
        expect_found: true,
        expect_dirs: "",
        expect_wps: (3, 8),
        rationale: "East blocked by 4x3 block at x=20..23 — wall-follow around N or S edge",
    },
    TestCase {
        name: "Long wall",
        start: (55, 50),
        goal: (55, 60),
        expect_found: true,
        expect_dirs: "",
        expect_wps: (3, 8),
        rationale: "South blocked by wall z=55 x=45..70 — follow to east or west end, then south",
    },
    TestCase {
        name: "U-trap",
        start: (63, 70),
        goal: (63, 95),
        expect_found: true,
        expect_dirs: "",
        expect_wps: (3, 12),
        rationale: "South into U, bottom wall blocks at z=90 — must escape through side, go around",
    },
    TestCase {
        name: "Corridor",
        start: (15, 80),
        goal: (35, 80),
        expect_found: true,
        expect_dirs: "",
        expect_wps: (3, 10),
        rationale: "Two offset vertical walls — zigzag through gap at z=82 then gap at z=77",
    },
    TestCase {
        name: "Enclosed",
        start: (80, 43),
        goal: (90, 43),
        expect_found: false,
        expect_dirs: "",
        expect_wps: (0, 0),
        rationale: "Goal inside solid 11x11 box — unreachable",
    },
    TestCase {
        name: "L-wall corner",
        start: (48, 25),
        goal: (55, 25),
        expect_found: true,
        expect_dirs: "",
        expect_wps: (3, 8),
        rationale: "East blocked by vertical wall x=50 — follow around L-corner",
    },
    TestCase {
        name: "C-shape",
        start: (12, 110),
        goal: (25, 110),
        expect_found: true,
        expect_dirs: "",
        expect_wps: (3, 10),
        rationale: "East blocked by C's vertical wall x=18 — follow around top or bottom of C",
    },
    TestCase {
        name: "Map edge",
        start: (3, 55),
        goal: (3, 70),
        expect_found: true,
        expect_dirs: "",
        expect_wps: (3, 6),
        rationale: "South blocked by wall at x=3 z=62..63 — small detour near map edge",
    },
];

// ──────────────────────────────────────────────────────────────────────────────
// Terrain textures
// ──────────────────────────────────────────────────────────────────────────────

fn heightmap_to_rgba(landscape: &Landscape<128>) -> Vec<u8> {
    let mut rgba = vec![0u8; MAP_SIZE * MAP_SIZE * 4];
    for z in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            let h = landscape.height[z][x];
            let idx = (z * MAP_SIZE + x) * 4;
            if h == 0 {
                rgba[idx] = 20;
                rgba[idx + 1] = 40;
                rgba[idx + 2] = 100;
            } else {
                let norm = (h as f32 / 1024.0).min(1.0);
                let r = (40.0 + norm * 180.0) as u8;
                let g = (100.0 + (1.0 - norm) * 100.0) as u8;
                let b = (30.0 + norm * 40.0) as u8;
                rgba[idx] = r;
                rgba[idx + 1] = g;
                rgba[idx + 2] = b;
            }
            rgba[idx + 3] = 255;
        }
    }
    rgba
}

/// Generate terrain texture from a RegionMap (for test mode).
/// Walkable = light green, unwalkable = dark gray.
fn region_map_to_rgba(region_map: &RegionMap) -> Vec<u8> {
    let mut rgba = vec![0u8; MAP_SIZE * MAP_SIZE * 4];
    for z in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            let tile = TileCoord::new((x * 2) as u8, (z * 2) as u8);
            let idx = (z * MAP_SIZE + x) * 4;
            if region_map.is_walkable(tile) {
                rgba[idx] = 140;
                rgba[idx + 1] = 180;
                rgba[idx + 2] = 100;
            } else {
                rgba[idx] = 40;
                rgba[idx + 1] = 40;
                rgba[idx + 2] = 50;
            }
            rgba[idx + 3] = 255;
        }
    }
    rgba
}

// ──────────────────────────────────────────────────────────────────────────────
// Region map builders
// ──────────────────────────────────────────────────────────────────────────────

fn set_wall(map: &mut RegionMap, x: usize, z: usize) {
    let tile = TileCoord::new((x * 2) as u8, (z * 2) as u8);
    map.get_cell_mut(tile).terrain_type = 1;
}

/// Build synthetic test map with obstacles exercising every pathfinder phase.
///
/// Obstacle layout (cell coords):
///   ③ Block 4×3      (20,50)→(23,52)     — simple wall-follow
///   ④ Long wall       z=55, x=45..70      — long follow around end
///   ⑤ U-trap          x=60/66, z=72..90   — concave trap (open top)
///   ⑥ Corridor        offset vertical walls — zigzag through gaps
///   ⑦ Enclosed box    (85,38)→(95,48)     — NOT_FOUND test
///   ⑧ L-wall          x=50 z=18..35 + z=18 x=50..62 — corner following
///   ⑨ C-shape         three walls forming C — wall-end detection
///   ⑩ Edge wall       x=3, z=62..63       — boundary test
fn build_test_map() -> RegionMap {
    let mut map = RegionMap::new();
    map.set_terrain_flags(1, 0x00); // terrain class 1 = wall (unwalkable)

    // ③ Block 4×3 at (20,50)→(23,52)
    for z in 50..=52 {
        for x in 20..=23 {
            set_wall(&mut map, x, z);
        }
    }

    // ④ Long wall at z=55, x=45..=70 (26 cells)
    for x in 45..=70 {
        set_wall(&mut map, x, 55);
    }

    // ⑤ U-trap: left/right walls + bottom, open top
    for z in 72..=90 {
        set_wall(&mut map, 60, z);
        set_wall(&mut map, 66, z);
    }
    for x in 60..=66 {
        set_wall(&mut map, x, 90);
    }

    // ⑥ Corridor: two offset vertical walls with gaps
    // Wall A at x=20: gap at z=82-83
    for z in 74..=81 {
        set_wall(&mut map, 20, z);
    }
    for z in 84..=86 {
        set_wall(&mut map, 20, z);
    }
    // Wall B at x=28: gap at z=77-78
    for z in 74..=76 {
        set_wall(&mut map, 28, z);
    }
    for z in 79..=86 {
        set_wall(&mut map, 28, z);
    }

    // ⑦ Enclosed box (85,38)→(95,48), solid fill
    for z in 38..=48 {
        for x in 85..=95 {
            set_wall(&mut map, x, z);
        }
    }

    // ⑧ L-wall: vertical x=50 z=18..=35, horizontal x=50..=62 z=18
    for z in 18..=35 {
        set_wall(&mut map, 50, z);
    }
    for x in 50..=62 {
        set_wall(&mut map, x, 18);
    }

    // ⑨ C-shape (open left): top/bottom horizontals + right vertical
    for x in 10..=18 {
        set_wall(&mut map, x, 105);
    }
    for z in 105..=120 {
        set_wall(&mut map, 18, z);
    }
    for x in 10..=18 {
        set_wall(&mut map, x, 120);
    }

    // ⑩ Small wall near map edge for boundary test
    for z in 62..=63 {
        set_wall(&mut map, 3, z);
    }

    map
}

/// Find the centroid of all land cells (height > 0) in world coordinates.
fn find_land_center(landscape: &Landscape<128>) -> [f32; 2] {
    let mut sum_x: f64 = 0.0;
    let mut sum_z: f64 = 0.0;
    let mut count: f64 = 0.0;
    for z in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            if landscape.height[z][x] > 0 {
                sum_x += x as f64;
                sum_z += z as f64;
                count += 1.0;
            }
        }
    }
    if count == 0.0 {
        return [WORLD_SIZE / 2.0, WORLD_SIZE / 2.0];
    }
    [
        (sum_x / count) as f32 * CELL_SIZE + CELL_SIZE / 2.0,
        (sum_z / count) as f32 * CELL_SIZE + CELL_SIZE / 2.0,
    ]
}

/// Find a suitable location for a building: a w×h block of walkable land.
fn find_building_site(
    landscape: &Landscape<128>,
    center_x: usize,
    center_z: usize,
    w: usize,
    h: usize,
) -> Option<(usize, usize)> {
    for radius in 0i32..64 {
        for dz in -radius..=radius {
            for dx in -radius..=radius {
                if dx.abs() != radius && dz.abs() != radius {
                    continue;
                }
                let bx = center_x as i32 + dx;
                let bz = center_z as i32 + dz;
                if bx < 2
                    || bz < 2
                    || bx + w as i32 >= MAP_SIZE as i32 - 2
                    || bz + h as i32 >= MAP_SIZE as i32 - 2
                {
                    continue;
                }
                let mut all_land = true;
                for oz in 0..h {
                    for ox in 0..w {
                        if landscape.height[bz as usize + oz][bx as usize + ox] == 0 {
                            all_land = false;
                            break;
                        }
                    }
                    if !all_land {
                        break;
                    }
                }
                if all_land {
                    return Some((bx as usize, bz as usize));
                }
            }
        }
    }
    None
}

/// Build region map from game landscape. Returns (map, camera_center).
fn build_region_map(landscape: &Landscape<128>) -> (RegionMap, [f32; 2]) {
    let mut map = RegionMap::new();
    map.set_terrain_flags(1, 0x00); // water
    map.set_terrain_flags(2, 0x00); // building

    for z in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            let tile = TileCoord::new((x * 2) as u8, (z * 2) as u8);
            if landscape.height[z][x] == 0 {
                map.get_cell_mut(tile).terrain_type = 1;
            }
        }
    }

    let land_center = find_land_center(landscape);
    let center_cell_x = (land_center[0] / CELL_SIZE) as usize;
    let center_cell_z = (land_center[1] / CELL_SIZE) as usize;
    let camera_center;
    if let Some((bx, bz)) = find_building_site(landscape, center_cell_x, center_cell_z, 4, 3) {
        for oz in 0..3usize {
            for ox in 0..4usize {
                let tile = TileCoord::new(((bx + ox) * 2) as u8, ((bz + oz) * 2) as u8);
                map.get_cell_mut(tile).terrain_type = 2;
                map.get_cell_mut(tile).flags_high = CELL_HAS_BUILDING;
            }
        }
        camera_center = [(bx as f32 + 2.0) * CELL_SIZE, (bz as f32 + 1.5) * CELL_SIZE];
        println!("Building placed at cell ({}, {}), size 4×3", bx, bz);
    } else {
        camera_center = land_center;
    }

    (map, camera_center)
}

// ──────────────────────────────────────────────────────────────────────────────
// Path analysis helpers
// ──────────────────────────────────────────────────────────────────────────────

fn wp_to_cell(wp: &Waypoint) -> (i32, i32) {
    (wp.tile_x as i32 >> 1, wp.tile_z as i32 >> 1)
}

fn direction_label(from: (i32, i32), to: (i32, i32)) -> &'static str {
    let dx = (to.0 - from.0).signum();
    let dz = (to.1 - from.1).signum();
    match (dx, dz) {
        (1, 0) => "E",
        (-1, 0) => "W",
        (0, 1) => "S",
        (0, -1) => "N",
        (1, 1) => "SE",
        (1, -1) => "NE",
        (-1, 1) => "SW",
        (-1, -1) => "NW",
        _ => "?",
    }
}

fn extract_directions(wps: &[Waypoint]) -> String {
    wps.windows(2)
        .map(|pair| direction_label(wp_to_cell(&pair[0]), wp_to_cell(&pair[1])))
        .collect::<Vec<_>>()
        .join(",")
}

fn format_waypoint_chain(wps: &[Waypoint]) -> String {
    if wps.is_empty() {
        return String::new();
    }
    let mut s = String::new();
    let first = wp_to_cell(&wps[0]);
    s.push_str(&format!("({},{})", first.0, first.1));
    for pair in wps.windows(2) {
        let from = wp_to_cell(&pair[0]);
        let to = wp_to_cell(&pair[1]);
        let dir = direction_label(from, to);
        s.push_str(&format!(" -{}-> ({},{})", dir, to.0, to.1));
    }
    s
}

fn winning_arm(debug: &PathfindDebug, goal_cell: (i32, i32)) -> &'static str {
    let dist = |trace: &[(i32, i32)]| -> i32 {
        trace.last().map_or(i32::MAX, |&(x, z)| {
            (x - goal_cell.0).abs() + (z - goal_cell.1).abs()
        })
    };
    if dist(&debug.arm0_trace) <= dist(&debug.arm1_trace) {
        "arm0 (right-hand)"
    } else {
        "arm1 (left-hand)"
    }
}

fn verify_test_case(tc: &TestCase, debug: &PathfindDebug) -> (Vec<&'static str>, Vec<String>) {
    let mut pass = Vec::new();
    let mut fail: Vec<String> = Vec::new();

    let found = matches!(debug.result, PathfindResult::Found(_));
    if found == tc.expect_found {
        pass.push(if found { "found=ok" } else { "not_found=ok" });
    } else {
        fail.push(format!(
            "expected {}, got {}",
            if tc.expect_found {
                "FOUND"
            } else {
                "NOT_FOUND"
            },
            if found { "FOUND" } else { "NOT_FOUND" }
        ));
    }

    if let PathfindResult::Found(ref wps) = debug.result {
        // Waypoint count
        if wps.len() >= tc.expect_wps.0 && wps.len() <= tc.expect_wps.1 {
            pass.push("wps=ok");
        } else {
            fail.push(format!(
                "expected {}-{} waypoints, got {}",
                tc.expect_wps.0,
                tc.expect_wps.1,
                wps.len()
            ));
        }

        // Direction check
        if !tc.expect_dirs.is_empty() {
            let dirs = extract_directions(wps);
            let mut dirs_ok = true;
            for expected in tc.expect_dirs.split(',') {
                if !dirs.contains(expected.trim()) {
                    fail.push(format!(
                        "expected dir {} in path, got: {}",
                        expected.trim(),
                        dirs
                    ));
                    dirs_ok = false;
                }
            }
            if dirs_ok {
                pass.push("dirs=ok");
            }
        }

        // Endpoint proximity
        if let (Some(first), Some(last)) = (wps.first(), wps.last()) {
            let fc = wp_to_cell(first);
            let lc = wp_to_cell(last);
            let sc = (tc.start.0 as i32, tc.start.1 as i32);
            let gc = (tc.goal.0 as i32, tc.goal.1 as i32);
            let sd = (fc.0 - sc.0).abs() + (fc.1 - sc.1).abs();
            let gd = (lc.0 - gc.0).abs() + (lc.1 - gc.1).abs();
            if sd <= 2 && gd <= 2 {
                pass.push("endpoints=ok");
            } else {
                if sd > 2 {
                    fail.push(format!("start wp {:?} far from {:?}", fc, sc));
                }
                if gd > 2 {
                    fail.push(format!("goal wp {:?} far from {:?}", lc, gc));
                }
            }
        }
    }

    (pass, fail)
}

fn print_pathfind_result(debug: &PathfindDebug, goal_cell: (i32, i32)) {
    let a0 = debug.arm0_trace.len();
    let a1 = debug.arm1_trace.len();
    match &debug.result {
        PathfindResult::Found(wps) => {
            let arm = winning_arm(debug, goal_cell);
            println!(
                "  Result:    FOUND via {} | arm0: {}, arm1: {}",
                arm, a0, a1
            );
            println!("  Path:      {}", format_waypoint_chain(wps));
            println!("  Dirs:      {}", extract_directions(wps));
        }
        PathfindResult::NotFound => {
            println!("  Result:    NOT FOUND | arm0: {}, arm1: {}", a0, a1);
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Simulation state
// ──────────────────────────────────────────────────────────────────────────────

struct Simulation {
    region_map: RegionMap,
    camera_center: [f32; 2],
    start: Option<TileCoord>,
    goal: Option<TileCoord>,
    debug: Option<PathfindDebug>,
    show_grid: bool,
}

impl Simulation {
    fn for_test_map() -> Self {
        Self {
            region_map: build_test_map(),
            camera_center: [WORLD_SIZE / 2.0, WORLD_SIZE / 2.0],
            start: None,
            goal: None,
            debug: None,
            show_grid: true,
        }
    }

    fn for_level(landscape: &Landscape<128>) -> Self {
        let (region_map, camera_center) = build_region_map(landscape);
        Self {
            region_map,
            camera_center,
            start: None,
            goal: None,
            debug: None,
            show_grid: true,
        }
    }

    fn set_start(&mut self, cell_x: usize, cell_z: usize) {
        let tx = ((cell_x * 2) as u8) & 0xFE;
        let tz = ((cell_z * 2) as u8) & 0xFE;
        self.start = Some(TileCoord::new(tx, tz));
        self.debug = None;
        println!(
            "Start: cell ({}, {}), tile ({:#x}, {:#x})",
            cell_x, cell_z, tx, tz
        );
        self.try_pathfind();
    }

    fn set_goal(&mut self, cell_x: usize, cell_z: usize) {
        let tx = ((cell_x * 2) as u8) & 0xFE;
        let tz = ((cell_z * 2) as u8) & 0xFE;
        self.goal = Some(TileCoord::new(tx, tz));
        println!(
            "Goal: cell ({}, {}), tile ({:#x}, {:#x})",
            cell_x, cell_z, tx, tz
        );
        self.try_pathfind();
    }

    fn set_test_case(&mut self, start: (usize, usize), goal: (usize, usize)) {
        let sx = ((start.0 * 2) as u8) & 0xFE;
        let sz = ((start.1 * 2) as u8) & 0xFE;
        self.start = Some(TileCoord::new(sx, sz));
        let gx = ((goal.0 * 2) as u8) & 0xFE;
        let gz = ((goal.1 * 2) as u8) & 0xFE;
        self.goal = Some(TileCoord::new(gx, gz));
        self.debug = None;
        self.try_pathfind();
    }

    fn try_pathfind(&mut self) {
        if let (Some(start), Some(goal)) = (self.start, self.goal) {
            let debug = movement::pathfind_debug(&self.region_map, start, goal);
            let goal_cell = ((goal.x as i32) >> 1, (goal.z as i32) >> 1);
            print_pathfind_result(&debug, goal_cell);
            self.debug = Some(debug);
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Overlay geometry builders
// ──────────────────────────────────────────────────────────────────────────────

/// Push a filled quad (2 triangles) for a cell at (cx, cz) in cell-space.
fn push_cell_quad(verts: &mut Vec<OverlayVertex>, cx: f32, cz: f32, color: [f32; 4]) {
    let x0 = cx * CELL_SIZE;
    let z0 = cz * CELL_SIZE;
    let x1 = x0 + CELL_SIZE;
    let z1 = z0 + CELL_SIZE;
    verts.push(OverlayVertex {
        position: [x0, z0],
        color,
    });
    verts.push(OverlayVertex {
        position: [x1, z0],
        color,
    });
    verts.push(OverlayVertex {
        position: [x1, z1],
        color,
    });
    verts.push(OverlayVertex {
        position: [x0, z0],
        color,
    });
    verts.push(OverlayVertex {
        position: [x1, z1],
        color,
    });
    verts.push(OverlayVertex {
        position: [x0, z1],
        color,
    });
}

fn build_cell_overlay(sim: &Simulation) -> Vec<OverlayVertex> {
    let mut verts = Vec::new();

    if let Some(debug) = &sim.debug {
        let arm0_color = [0.0, 0.8, 0.9, 0.25];
        let arm1_color = [0.9, 0.2, 0.8, 0.25];
        let visited_color = [0.5, 0.5, 0.5, 0.15];

        let arm0_set: std::collections::HashSet<(i32, i32)> =
            debug.arm0_trace.iter().copied().collect();
        let arm1_set: std::collections::HashSet<(i32, i32)> =
            debug.arm1_trace.iter().copied().collect();

        for z in 0..REGION_GRID_SIZE as i32 {
            for x in 0..REGION_GRID_SIZE as i32 {
                if debug.visited.is_visited(x, z) {
                    let color = if arm0_set.contains(&(x, z)) && arm1_set.contains(&(x, z)) {
                        [0.5, 0.5, 0.9, 0.3]
                    } else if arm0_set.contains(&(x, z)) {
                        arm0_color
                    } else if arm1_set.contains(&(x, z)) {
                        arm1_color
                    } else {
                        visited_color
                    };
                    push_cell_quad(&mut verts, x as f32, z as f32, color);
                }
            }
        }

        // Arm 0 trace line (cyan)
        for pair in debug.arm0_trace.windows(2) {
            let (x0, z0) = pair[0];
            let (x1, z1) = pair[1];
            push_line_quad(
                &mut verts,
                (x0 as f32 + 0.5) * CELL_SIZE,
                (z0 as f32 + 0.5) * CELL_SIZE,
                (x1 as f32 + 0.5) * CELL_SIZE,
                (z1 as f32 + 0.5) * CELL_SIZE,
                CELL_SIZE * 0.08,
                [0.0, 1.0, 1.0, 0.8],
            );
        }
        // Arm 1 trace line (magenta)
        for pair in debug.arm1_trace.windows(2) {
            let (x0, z0) = pair[0];
            let (x1, z1) = pair[1];
            push_line_quad(
                &mut verts,
                (x0 as f32 + 0.5) * CELL_SIZE,
                (z0 as f32 + 0.5) * CELL_SIZE,
                (x1 as f32 + 0.5) * CELL_SIZE,
                (z1 as f32 + 0.5) * CELL_SIZE,
                CELL_SIZE * 0.08,
                [1.0, 0.2, 0.8, 0.8],
            );
        }

        // Final path (yellow)
        if let PathfindResult::Found(ref wps) = debug.result {
            for pair in wps.windows(2) {
                let x0 = (pair[0].tile_x as f32 + 1.0) * (CELL_SIZE / 2.0);
                let z0 = (pair[0].tile_z as f32 + 1.0) * (CELL_SIZE / 2.0);
                let x1 = (pair[1].tile_x as f32 + 1.0) * (CELL_SIZE / 2.0);
                let z1 = (pair[1].tile_z as f32 + 1.0) * (CELL_SIZE / 2.0);
                push_line_quad(
                    &mut verts,
                    x0,
                    z0,
                    x1,
                    z1,
                    CELL_SIZE * 0.12,
                    [1.0, 0.9, 0.0, 0.9],
                );
            }
            for wp in wps {
                let cx = (wp.tile_x as f32 + 1.0) * (CELL_SIZE / 2.0);
                let cz = (wp.tile_z as f32 + 1.0) * (CELL_SIZE / 2.0);
                let s = CELL_SIZE * 0.15;
                push_cell_quad_centered(&mut verts, cx, cz, s, [1.0, 1.0, 0.0, 0.9]);
            }
        }
    }

    // Start marker (green)
    if let Some(start) = sim.start {
        let cx = (start.x as f32 / 2.0 + 0.5) * CELL_SIZE;
        let cz = (start.z as f32 / 2.0 + 0.5) * CELL_SIZE;
        push_cell_quad_centered(&mut verts, cx, cz, CELL_SIZE * 0.3, [0.0, 1.0, 0.0, 0.9]);
    }

    // Goal marker (red)
    if let Some(goal) = sim.goal {
        let cx = (goal.x as f32 / 2.0 + 0.5) * CELL_SIZE;
        let cz = (goal.z as f32 / 2.0 + 0.5) * CELL_SIZE;
        push_cell_quad_centered(&mut verts, cx, cz, CELL_SIZE * 0.3, [1.0, 0.2, 0.2, 0.9]);
    }

    verts
}

fn build_grid_overlay(sim: &Simulation) -> Vec<OverlayVertex> {
    if !sim.show_grid {
        return Vec::new();
    }

    let mut verts = Vec::new();
    let grid_color = [1.0, 1.0, 1.0, 0.08];
    let thickness = CELL_SIZE * 0.02;

    for x in 0..=MAP_SIZE {
        let wx = x as f32 * CELL_SIZE;
        push_line_quad(&mut verts, wx, 0.0, wx, WORLD_SIZE, thickness, grid_color);
    }
    for z in 0..=MAP_SIZE {
        let wz = z as f32 * CELL_SIZE;
        push_line_quad(&mut verts, 0.0, wz, WORLD_SIZE, wz, thickness, grid_color);
    }

    // Unwalkable cell overlay — water vs building
    let water_color = [0.0, 0.0, 0.0, 0.4];
    let building_color = [0.6, 0.35, 0.1, 0.7];
    for z in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            let tile = TileCoord::new((x * 2) as u8, (z * 2) as u8);
            if !sim.region_map.is_walkable(tile) {
                let tc = sim.region_map.terrain_class(tile);
                let color = if tc == 2 { building_color } else { water_color };
                push_cell_quad(&mut verts, x as f32, z as f32, color);
            }
        }
    }

    verts
}

/// Push a thin quad along a line segment (for rendering lines via triangles).
fn push_line_quad(
    verts: &mut Vec<OverlayVertex>,
    x0: f32,
    z0: f32,
    x1: f32,
    z1: f32,
    thickness: f32,
    color: [f32; 4],
) {
    let dx = x1 - x0;
    let dz = z1 - z0;
    let len = (dx * dx + dz * dz).sqrt().max(0.001);
    let nx = -dz / len * thickness;
    let nz = dx / len * thickness;

    let a = [x0 + nx, z0 + nz];
    let b = [x0 - nx, z0 - nz];
    let c = [x1 - nx, z1 - nz];
    let d = [x1 + nx, z1 + nz];

    verts.push(OverlayVertex { position: a, color });
    verts.push(OverlayVertex { position: b, color });
    verts.push(OverlayVertex { position: c, color });
    verts.push(OverlayVertex { position: a, color });
    verts.push(OverlayVertex { position: c, color });
    verts.push(OverlayVertex { position: d, color });
}

/// Push a centered square quad.
fn push_cell_quad_centered(
    verts: &mut Vec<OverlayVertex>,
    cx: f32,
    cz: f32,
    half_size: f32,
    color: [f32; 4],
) {
    let x0 = cx - half_size;
    let z0 = cz - half_size;
    let x1 = cx + half_size;
    let z1 = cz + half_size;
    verts.push(OverlayVertex {
        position: [x0, z0],
        color,
    });
    verts.push(OverlayVertex {
        position: [x1, z0],
        color,
    });
    verts.push(OverlayVertex {
        position: [x1, z1],
        color,
    });
    verts.push(OverlayVertex {
        position: [x0, z0],
        color,
    });
    verts.push(OverlayVertex {
        position: [x1, z1],
        color,
    });
    verts.push(OverlayVertex {
        position: [x0, z1],
        color,
    });
}

// ──────────────────────────────────────────────────────────────────────────────
// Camera
// ──────────────────────────────────────────────────────────────────────────────

struct Camera {
    center: [f32; 2],
    zoom: f32,
}

impl Camera {
    fn for_position(center: [f32; 2]) -> Self {
        Self { center, zoom: 3.5 }
    }

    fn projection(&self, screen_w: f32, screen_h: f32) -> [[f32; 4]; 4] {
        let hw = screen_w * self.zoom / 2.0;
        let hh = screen_h * self.zoom / 2.0;
        let l = self.center[0] - hw;
        let r = self.center[0] + hw;
        let b = self.center[1] + hh;
        let t = self.center[1] - hh;
        [
            [2.0 / (r - l), 0.0, 0.0, 0.0],
            [0.0, 2.0 / (t - b), 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-(r + l) / (r - l), -(t + b) / (t - b), 0.0, 1.0],
        ]
    }

    fn screen_to_world(&self, sx: f32, sy: f32, screen_w: f32, screen_h: f32) -> [f32; 2] {
        let ndc_x = (sx / screen_w) * 2.0 - 1.0;
        let ndc_y = (sy / screen_h) * 2.0 - 1.0;
        let hw = screen_w * self.zoom / 2.0;
        let hh = screen_h * self.zoom / 2.0;
        [self.center[0] + ndc_x * hw, self.center[1] + ndc_y * hh]
    }

    fn world_to_cell(&self, world: [f32; 2]) -> Option<(usize, usize)> {
        let cx = (world[0] / CELL_SIZE) as i32;
        let cz = (world[1] / CELL_SIZE) as i32;
        if cx >= 0 && cx < MAP_SIZE as i32 && cz >= 0 && cz < MAP_SIZE as i32 {
            Some((cx as usize, cz as usize))
        } else {
            None
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Application
// ──────────────────────────────────────────────────────────────────────────────

struct App {
    window: Option<Arc<Window>>,
    state: Option<ViewerState>,
    base_path: PathBuf,
    mode: MapMode,
    sim: Simulation,
    landscape: Option<Landscape<128>>,
}

struct ViewerState {
    gpu: GpuContext,
    terrain_pipeline: wgpu::RenderPipeline,
    cell_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    terrain_texture: GpuTexture,
    sampler: wgpu::Sampler,
    uniform_buffer: GpuBuffer,
    terrain_vertex_buffer: GpuBuffer,
    cell_buffer: GpuBuffer,
    cell_vert_count: u32,
    camera: Camera,
    cursor_pos: [f32; 2],
}

impl App {
    fn new(base_path: PathBuf) -> Self {
        let sim = Simulation::for_test_map();
        Self {
            window: None,
            state: None,
            base_path,
            mode: MapMode::Test,
            sim,
            landscape: None,
        }
    }

    fn terrain_rgba(&self) -> Vec<u8> {
        match self.mode {
            MapMode::Test => region_map_to_rgba(&self.sim.region_map),
            MapMode::Level(_) => heightmap_to_rgba(self.landscape.as_ref().unwrap()),
        }
    }

    fn rebuild_gpu(&mut self) {
        let rgba = self.terrain_rgba();
        if let Some(state) = &mut self.state {
            state.terrain_texture = GpuTexture::new_2d(
                &state.gpu.device,
                &state.gpu.queue,
                MAP_SIZE as u32,
                MAP_SIZE as u32,
                wgpu::TextureFormat::Rgba8UnormSrgb,
                &rgba,
                "terrain_texture",
            );
            state.bind_group = state
                .gpu
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("pathfinding_bg"),
                    layout: &state.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: state.uniform_buffer.buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(
                                &state.terrain_texture.view,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&state.sampler),
                        },
                    ],
                });
            state.camera = Camera::for_position(self.sim.camera_center);
        }
    }

    fn switch_to_test(&mut self) {
        self.mode = MapMode::Test;
        self.sim = Simulation::for_test_map();
        self.landscape = None;
        self.rebuild_gpu();
        self.update_title();
        println!("Switched to test map (keys 1-9, 0 to select test cases)");
    }

    fn switch_to_level(&mut self, level_num: u8) {
        let level = LevelRes::new(&self.base_path, level_num, None);
        self.sim = Simulation::for_level(&level.landscape);
        self.landscape = Some(level.landscape);
        self.mode = MapMode::Level(level_num);
        self.rebuild_gpu();
        self.update_title();
        println!(
            "Loaded level {} (N/P to switch, click to set start/goal)",
            level_num
        );
    }

    fn update_title(&self) {
        if let Some(window) = &self.window {
            let title = match self.mode {
                MapMode::Test => "Pathfinding Demo — Test Map".to_string(),
                MapMode::Level(n) => format!("Pathfinding Demo — Level {}", n),
            };
            window.set_title(&title);
        }
    }

    fn select_test_case(&mut self, idx: usize) {
        if idx >= TEST_CASES.len() {
            return;
        }
        let tc = &TEST_CASES[idx];
        self.sim.set_test_case(tc.start, tc.goal);

        // Auto-center camera on midpoint with adaptive zoom
        let mid_x = ((tc.start.0 + tc.goal.0) as f32 / 2.0 + 0.5) * CELL_SIZE;
        let mid_z = ((tc.start.1 + tc.goal.1) as f32 / 2.0 + 0.5) * CELL_SIZE;
        if let Some(state) = &mut self.state {
            let dx = (tc.start.0 as f32 - tc.goal.0 as f32).abs() + 12.0;
            let dz = (tc.start.1 as f32 - tc.goal.1 as f32).abs() + 12.0;
            let span = dx.max(dz) * CELL_SIZE;
            let screen = state.gpu.size.width.min(state.gpu.size.height) as f32;
            state.camera.center = [mid_x, mid_z];
            state.camera.zoom = (span / screen).max(2.0);
        }

        // Enhanced verifiable output
        println!(
            "[Case {}] {}: ({},{}) -> ({},{})",
            idx + 1,
            tc.name,
            tc.start.0,
            tc.start.1,
            tc.goal.0,
            tc.goal.1
        );
        println!("  Rationale: {}", tc.rationale);

        if let Some(debug) = &self.sim.debug {
            let goal_cell = (tc.goal.0 as i32, tc.goal.1 as i32);
            print_pathfind_result(debug, goal_cell);
            let (pass, fail) = verify_test_case(tc, debug);
            if fail.is_empty() {
                println!("  Checks:    PASS ({})", pass.join(", "));
            } else {
                println!("  Checks:    FAIL — {}", fail.join("; "));
                if !pass.is_empty() {
                    println!("             passed: {}", pass.join(", "));
                }
            }
        } else {
            println!("  Result:    NO DEBUG DATA");
            println!("  Checks:    FAIL — pathfinder did not run");
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let title = match self.mode {
            MapMode::Test => "Pathfinding Demo — Test Map".to_string(),
            MapMode::Level(n) => format!("Pathfinding Demo — Level {}", n),
        };

        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title(title)
                        .with_inner_size(winit::dpi::LogicalSize::new(1200, 1200)),
                )
                .unwrap(),
        );
        self.window = Some(window.clone());

        let gpu = pollster::block_on(GpuContext::new(window));
        let device = &gpu.device;

        let rgba = self.terrain_rgba();
        let terrain_texture = GpuTexture::new_2d(
            device,
            &gpu.queue,
            MAP_SIZE as u32,
            MAP_SIZE as u32,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            &rgba,
            "terrain_texture",
        );
        let sampler = GpuTexture::create_sampler(device, true);
        let uniform_buffer = GpuBuffer::new_uniform(device, 64, "pathfinding_uniforms");

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("pathfinding_bgl"),
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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("pathfinding_bg"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&terrain_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let shader_source = include_str!("../../shaders/pathfinding_demo.wgsl");

        let terrain_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("terrain_shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pathfinding_layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        let terrain_vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TerrainVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        };

        let terrain_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("terrain_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &terrain_shader,
                entry_point: Some("vs_terrain"),
                buffers: &[terrain_vertex_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &terrain_shader,
                entry_point: Some("fs_terrain"),
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let overlay_vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<OverlayVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        };

        let overlay_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("overlay_shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let cell_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("cell_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &overlay_shader,
                entry_point: Some("vs_overlay"),
                buffers: &[overlay_vertex_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &overlay_shader,
                entry_point: Some("fs_overlay"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: gpu.surface_format(),
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

        let terrain_quad = [
            TerrainVertex {
                position: [0.0, 0.0],
                uv: [0.0, 0.0],
            },
            TerrainVertex {
                position: [WORLD_SIZE, 0.0],
                uv: [1.0, 0.0],
            },
            TerrainVertex {
                position: [WORLD_SIZE, WORLD_SIZE],
                uv: [1.0, 1.0],
            },
            TerrainVertex {
                position: [0.0, 0.0],
                uv: [0.0, 0.0],
            },
            TerrainVertex {
                position: [WORLD_SIZE, WORLD_SIZE],
                uv: [1.0, 1.0],
            },
            TerrainVertex {
                position: [0.0, WORLD_SIZE],
                uv: [0.0, 1.0],
            },
        ];
        let terrain_vertex_buffer =
            GpuBuffer::new_vertex(device, bytemuck::cast_slice(&terrain_quad), "terrain_quad");

        let cell_buffer = GpuBuffer::new_vertex(
            device,
            &vec![0u8; MAX_OVERLAY_VERTS * std::mem::size_of::<OverlayVertex>()],
            "cell_overlay",
        );

        self.state = Some(ViewerState {
            gpu,
            terrain_pipeline,
            cell_pipeline,
            bind_group_layout,
            bind_group,
            terrain_texture,
            sampler,
            uniform_buffer,
            terrain_vertex_buffer,
            cell_buffer,
            cell_vert_count: 0,
            camera: Camera::for_position(self.sim.camera_center),
            cursor_pos: [0.0, 0.0],
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _wid: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(state) = &mut self.state {
                    state.gpu.resize(size);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(state) = &mut self.state {
                    state.cursor_pos = [position.x as f32, position.y as f32];
                }
            }
            WindowEvent::MouseInput {
                state: btn_state,
                button,
                ..
            } => {
                if btn_state == ElementState::Pressed {
                    if let Some(vs) = &self.state {
                        let sw = vs.gpu.size.width as f32;
                        let sh = vs.gpu.size.height as f32;
                        let world =
                            vs.camera
                                .screen_to_world(vs.cursor_pos[0], vs.cursor_pos[1], sw, sh);
                        if let Some((cx, cz)) = vs.camera.world_to_cell(world) {
                            match button {
                                MouseButton::Left => self.sim.set_start(cx, cz),
                                MouseButton::Right => self.sim.set_goal(cx, cz),
                                _ => {}
                            }
                        }
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    if let PhysicalKey::Code(key) = event.physical_key {
                        match key {
                            KeyCode::Escape => {
                                event_loop.exit();
                                return;
                            }
                            KeyCode::KeyG => {
                                self.sim.show_grid = !self.sim.show_grid;
                                println!("Grid: {}", if self.sim.show_grid { "ON" } else { "OFF" });
                            }
                            KeyCode::KeyR => {
                                match self.mode {
                                    MapMode::Test => self.sim = Simulation::for_test_map(),
                                    MapMode::Level(_) => {
                                        if let Some(ls) = &self.landscape {
                                            self.sim = Simulation::for_level(ls);
                                        }
                                    }
                                }
                                println!("Reset");
                            }
                            // Mode toggle
                            KeyCode::KeyM => match self.mode {
                                MapMode::Test => self.switch_to_level(3),
                                MapMode::Level(_) => self.switch_to_test(),
                            },
                            // Zoom
                            KeyCode::Equal | KeyCode::NumpadAdd => {
                                if let Some(state) = &mut self.state {
                                    state.camera.zoom = (state.camera.zoom * 0.8).max(0.5);
                                }
                            }
                            KeyCode::Minus | KeyCode::NumpadSubtract => {
                                if let Some(state) = &mut self.state {
                                    state.camera.zoom = (state.camera.zoom * 1.25).min(200.0);
                                }
                            }
                            // Pan
                            KeyCode::ArrowLeft => {
                                if let Some(state) = &mut self.state {
                                    state.camera.center[0] -= 200.0 * state.camera.zoom;
                                }
                            }
                            KeyCode::ArrowRight => {
                                if let Some(state) = &mut self.state {
                                    state.camera.center[0] += 200.0 * state.camera.zoom;
                                }
                            }
                            KeyCode::ArrowUp => {
                                if let Some(state) = &mut self.state {
                                    state.camera.center[1] -= 200.0 * state.camera.zoom;
                                }
                            }
                            KeyCode::ArrowDown => {
                                if let Some(state) = &mut self.state {
                                    state.camera.center[1] += 200.0 * state.camera.zoom;
                                }
                            }
                            // Level switching (level mode only)
                            KeyCode::KeyN => {
                                if let MapMode::Level(n) = self.mode {
                                    self.switch_to_level(n.wrapping_add(1).max(1));
                                }
                            }
                            KeyCode::KeyP => {
                                if let MapMode::Level(n) = self.mode {
                                    self.switch_to_level(if n <= 1 { 25 } else { n - 1 });
                                }
                            }
                            // Test case selection (test mode only)
                            KeyCode::Digit1 => {
                                if matches!(self.mode, MapMode::Test) {
                                    self.select_test_case(0);
                                }
                            }
                            KeyCode::Digit2 => {
                                if matches!(self.mode, MapMode::Test) {
                                    self.select_test_case(1);
                                }
                            }
                            KeyCode::Digit3 => {
                                if matches!(self.mode, MapMode::Test) {
                                    self.select_test_case(2);
                                }
                            }
                            KeyCode::Digit4 => {
                                if matches!(self.mode, MapMode::Test) {
                                    self.select_test_case(3);
                                }
                            }
                            KeyCode::Digit5 => {
                                if matches!(self.mode, MapMode::Test) {
                                    self.select_test_case(4);
                                }
                            }
                            KeyCode::Digit6 => {
                                if matches!(self.mode, MapMode::Test) {
                                    self.select_test_case(5);
                                }
                            }
                            KeyCode::Digit7 => {
                                if matches!(self.mode, MapMode::Test) {
                                    self.select_test_case(6);
                                }
                            }
                            KeyCode::Digit8 => {
                                if matches!(self.mode, MapMode::Test) {
                                    self.select_test_case(7);
                                }
                            }
                            KeyCode::Digit9 => {
                                if matches!(self.mode, MapMode::Test) {
                                    self.select_test_case(8);
                                }
                            }
                            KeyCode::Digit0 => {
                                if matches!(self.mode, MapMode::Test) {
                                    self.select_test_case(9);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(state) = &mut self.state {
                    state.render(&self.sim);
                }
            }
            _ => {}
        }

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl ViewerState {
    fn render(&mut self, sim: &Simulation) {
        let sw = self.gpu.size.width as f32;
        let sh = self.gpu.size.height as f32;

        let proj = self.camera.projection(sw, sh);
        self.gpu
            .queue
            .write_buffer(&self.uniform_buffer.buffer, 0, bytemuck::bytes_of(&proj));

        let output = match self.gpu.surface.get_current_texture() {
            Ok(t) => t,
            Err(_) => return,
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("pathfinding_encoder"),
            });

        let mut overlay_verts = build_grid_overlay(sim);
        overlay_verts.extend(build_cell_overlay(sim));
        self.cell_vert_count = (overlay_verts.len() as u32).min(MAX_OVERLAY_VERTS as u32);
        if self.cell_vert_count > 0 {
            self.gpu.queue.write_buffer(
                &self.cell_buffer.buffer,
                0,
                bytemuck::cast_slice(&overlay_verts[..self.cell_vert_count as usize]),
            );
        }

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("pathfinding_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.02,
                            g: 0.02,
                            b: 0.05,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            pass.set_pipeline(&self.terrain_pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_vertex_buffer(0, self.terrain_vertex_buffer.buffer.slice(..));
            pass.draw(0..6, 0..1);

            if self.cell_vert_count > 0 {
                pass.set_pipeline(&self.cell_pipeline);
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.set_vertex_buffer(0, self.cell_buffer.buffer.slice(..));
                pass.draw(0..self.cell_vert_count, 0..1);
            }
        }

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// CLI & main
// ──────────────────────────────────────────────────────────────────────────────

fn cli() -> Command {
    Command::new("pathfinding-demo")
        .about("Pathfinding visualizer for Populous: The Beginning")
        .arg(
            Arg::new("base")
                .long("base")
                .value_parser(clap::value_parser!(PathBuf))
                .help("Path to game install directory"),
        )
}

fn main() {
    let matches = cli().get_matches();

    let base = matches
        .get_one::<PathBuf>("base")
        .cloned()
        .unwrap_or_else(|| {
            let candidates = [
                PathBuf::from("data/original_game"),
                PathBuf::from("../data/original_game"),
            ];
            for c in &candidates {
                if c.join("levels").exists() {
                    return c.clone();
                }
            }
            PathBuf::from("data/original_game")
        });

    println!("Pathfinding Demo");
    println!("  Mode:        test map (M to toggle level mode)");
    println!("  1-9, 0:      select test case");
    println!("  Left-click:  set start");
    println!("  Right-click: set goal (triggers pathfinding)");
    println!("  G:           toggle grid");
    println!("  R:           reset");
    println!("  +/-:         zoom");
    println!("  Arrows:      pan");
    println!("  M:           toggle test/level mode");
    println!("  N/P:         next/prev level (level mode)");
    println!();
    println!("  Cyan cells:    arm 0 (right-hand wall-follow)");
    println!("  Magenta cells: arm 1 (left-hand wall-follow)");
    println!("  Yellow line:   final path");

    let env = env_logger::Env::default()
        .filter_or("F_LOG_LEVEL", "info")
        .write_style_or("F_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new(base);
    event_loop.run_app(&mut app).unwrap();
}
