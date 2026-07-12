// Full cascade after terrain modification.
// After height changes, updates: normals -> walkability -> water -> pathfinding -> mesh.

use super::water;
use crate::engine::movement::segment::SegmentPool;

const GRID_SIZE: usize = 128;
const GRID_MASK: usize = 127;

/// Steep slope threshold: if abs height difference between adjacent cells > 512,
/// mark as unwalkable.
const STEEP_SLOPE_THRESHOLD: u16 = 512;

/// Walkability flag for steep slope (unwalkable).
const STEEP_SLOPE_FLAG: u8 = 0x02;

/// Defines the affected cell range for a cascade update.
/// Coordinates are cell indices in the 128x128 grid.
/// Wrapping is handled internally by cascade functions.
#[derive(Debug, Clone)]
pub struct CascadeRegion {
    pub min_x: usize,
    pub min_y: usize,
    pub max_x: usize,
    pub max_y: usize,
}

impl CascadeRegion {
    /// Iterate over all cells in the region, handling toroidal wrapping.
    pub fn for_each_cell<F: FnMut(usize, usize)>(&self, mut f: F) {
        let width = if self.max_x >= self.min_x {
            self.max_x - self.min_x + 1
        } else {
            (GRID_SIZE - self.min_x) + self.max_x + 1
        };
        let height = if self.max_y >= self.min_y {
            self.max_y - self.min_y + 1
        } else {
            (GRID_SIZE - self.min_y) + self.max_y + 1
        };

        for dy in 0..height {
            for dx in 0..width {
                let x = (self.min_x + dx) & GRID_MASK;
                let y = (self.min_y + dy) & GRID_MASK;
                f(x, y);
            }
        }
    }

    /// Check if a tile coordinate (u8) falls within this region.
    /// Handles toroidal wrapping. Tile coords are divided by 2 to get cell coords
    /// since tiles step by 2 (the low bit is masked by 0xFE).
    pub fn contains_tile(&self, tile_x: u8, tile_z: u8) -> bool {
        // Convert tile to cell index (tiles are in 0-254 step 2, cells are 0-127)
        let cx = (tile_x >> 1) as usize;
        let cy = (tile_z >> 1) as usize;

        let in_x = if self.min_x <= self.max_x {
            cx >= self.min_x && cx <= self.max_x
        } else {
            // Wraps around
            cx >= self.min_x || cx <= self.max_x
        };

        let in_y = if self.min_y <= self.max_y {
            cy >= self.min_y && cy <= self.max_y
        } else {
            cy >= self.min_y || cy <= self.max_y
        };

        in_x && in_y
    }
}

/// Result of a terrain cascade operation.
#[derive(Debug)]
pub struct CascadeResult {
    pub normals_updated: bool,
    pub walkability_changed: Vec<(usize, usize)>,
    pub water_cells_changed: Vec<(usize, usize)>,
    pub segments_invalidated: u16,
    pub mesh_dirty: bool,
}

/// Recalculate normal for a single cell using cross product of adjacent height differences.
/// Normal = cross((h[y][x+1]-h[y][x-1], 2.0, 0), (0, 2.0, h[y+1][x]-h[y-1][x])), normalized.
fn calculate_normal(heights: &[[u16; GRID_SIZE]; GRID_SIZE], x: usize, y: usize) -> [f32; 3] {
    let xp = (x + 1) & GRID_MASK;
    let xm = (x.wrapping_sub(1)) & GRID_MASK;
    let yp = (y + 1) & GRID_MASK;
    let ym = (y.wrapping_sub(1)) & GRID_MASK;

    let dx = heights[y][xp] as f32 - heights[y][xm] as f32;
    let dz = heights[yp][x] as f32 - heights[ym][x] as f32;

    // Cross product of (dx, 2.0, 0.0) x (0.0, 2.0, dz)
    // = (2.0*dz - 0.0*2.0, 0.0*0.0 - dx*dz, dx*2.0 - 2.0*0.0)
    // = (2*dz, -dx*dz, 2*dx)
    // Wait, let me redo this properly:
    // a = (dx, 2.0, 0.0)
    // b = (0.0, 2.0, dz)
    // cross = (a.y*b.z - a.z*b.y, a.z*b.x - a.x*b.z, a.x*b.y - a.y*b.x)
    //       = (2.0*dz - 0.0*2.0, 0.0*0.0 - dx*dz, dx*2.0 - 2.0*0.0)
    //       = (2*dz, -dx*dz, 2*dx)
    // Hmm, that doesn't give (0, 1, 0) for flat terrain.
    //
    // For flat terrain: dx=0, dz=0 -> cross = (0, 0, 0). That's degenerate.
    // The standard approach: use tangent vectors along x and z axes.
    // Tangent_x = (2.0, dx, 0.0)  -- step of 2 cells in x, height difference dx
    // Tangent_z = (0.0, dz, 2.0)
    // Normal = Tangent_z x Tangent_x (order for outward-facing up)
    // = (dz*0 - 2*dx, 2*2 - 0*0, 0*dx - dz*0)... no.
    //
    // Let me think about this more carefully.
    // The surface at (x,y) with height h:
    // Position = (x, h(x,y), y) -- using Y-up convention
    // Tangent along X: (1, dh/dx, 0) ~ (2, dx, 0) with step size 2
    // Tangent along Y: (0, dh/dy, 1) ~ (0, dz, 2) with step size 2
    // Normal = Tangent_X x Tangent_Y = (dx*2 - 0*dz, 0*0 - 2*2, 2*dz - dx*0)
    //        = (2*0 - 0, ... )
    // Actually:
    // T_x = (2, dx, 0)
    // T_z = (0, dz, 2)
    // N = T_x x T_z = (dx*2 - 0*dz, 0*0 - 2*2, 2*dz - dx*0) = (2*dx, -4, 2*dz)
    // That points DOWN. Flip it:
    // N = T_z x T_x = (dz*0 - 2*dx, 2*0 - 0*2, 0*dx - dz*2)...
    //
    // Let me just compute directly:
    // T_x = (2, dx, 0), T_z = (0, dz, 2)
    // T_z x T_x:
    //   i: dz*0 - 2*dx = -2*dx
    //   j: 2*2 - 0*0 = 4
    //   k: 0*dx - dz*2 ... wait, 0*dx - dz*0 = 0... no:
    //   k: T_z.x * T_x.y - T_z.y * T_x.x = 0*dx - dz*2 = -2*dz
    // Hmm that gives (-2dx, 4, -2dz). For flat: (0, 4, 0) -> normalized (0, 1, 0). Good!

    let nx = -2.0 * dx;
    let ny = 4.0;
    let nz = -2.0 * dz;

    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    if len > 0.0001 {
        [nx / len, ny / len, nz / len]
    } else {
        [0.0, 1.0, 0.0]
    }
}

/// Check if a cell has a steep slope (any adjacent height difference > threshold).
fn is_steep_slope(heights: &[[u16; GRID_SIZE]; GRID_SIZE], x: usize, y: usize) -> bool {
    let h = heights[y][x];
    let neighbors = [
        heights[y][(x + 1) & GRID_MASK],
        heights[y][(x.wrapping_sub(1)) & GRID_MASK],
        heights[(y + 1) & GRID_MASK][x],
        heights[(y.wrapping_sub(1)) & GRID_MASK][x],
    ];

    for nh in neighbors {
        let diff = if h > nh { h - nh } else { nh - h };
        if diff > STEEP_SLOPE_THRESHOLD {
            return true;
        }
    }
    false
}

/// Run the full cascade after terrain height modification.
/// Steps in order:
/// 1. Recalculate normals for cells in region
/// 2. Update walkability flags (steep slopes become unwalkable)
/// 3. Update water cells
/// 4. Invalidate pathfinding segments in region
/// 5. Mark mesh as dirty
pub fn terrain_cascade(
    heights: &[[u16; GRID_SIZE]; GRID_SIZE],
    region: &CascadeRegion,
    normals: &mut [[[f32; 3]; GRID_SIZE]; GRID_SIZE],
    walkability: &mut [[u8; GRID_SIZE]; GRID_SIZE],
    water_level: u16,
    segment_pool: &mut SegmentPool,
) -> CascadeResult {
    // Step 1: Recalculate normals
    region.for_each_cell(|x, y| {
        normals[y][x] = calculate_normal(heights, x, y);
    });

    // Step 2: Update walkability (steep slopes)
    let mut walkability_changed = Vec::new();
    region.for_each_cell(|x, y| {
        let steep = is_steep_slope(heights, x, y);
        let was_steep = walkability[y][x] & STEEP_SLOPE_FLAG != 0;

        if steep && !was_steep {
            walkability[y][x] |= STEEP_SLOPE_FLAG;
            walkability_changed.push((x, y));
        } else if !steep && was_steep {
            walkability[y][x] &= !STEEP_SLOPE_FLAG;
            walkability_changed.push((x, y));
        }
    });

    // Step 3: Update water cells
    let water_cells_changed = water::update_water_cells(heights, walkability, region, water_level);

    // Step 4: Invalidate pathfinding segments
    let segments_invalidated = invalidate_segments_in_region(segment_pool, region);

    CascadeResult {
        normals_updated: true,
        walkability_changed,
        water_cells_changed,
        segments_invalidated,
        mesh_dirty: true,
    }
}

/// Invalidate cached path segments whose src or dst falls within the modified region.
/// Uses toroidal wrapping for coordinate comparison.
/// Returns count of invalidated segments.
pub fn invalidate_segments_in_region(
    segment_pool: &mut SegmentPool,
    region: &CascadeRegion,
) -> u16 {
    let mut count = 0u16;

    // Skip index 0 (sentinel)
    for i in 1..segment_pool.segments.len() {
        let seg = &segment_pool.segments[i];
        if seg.ref_count <= 0 {
            continue;
        }

        let src_in = region.contains_tile(seg.start_tile_x, seg.start_tile_z);
        let dst_in = region.contains_tile(seg.end_tile_x, seg.end_tile_z);

        if src_in || dst_in {
            segment_pool.segments[i].ref_count = 0;
            segment_pool.active_count -= 1;
            count += 1;
        }
    }

    count
}
