// Full cascade after terrain modification.
// After height changes, updates: normals -> walkability -> water -> pathfinding -> mesh.

use crate::engine::movement::segment::SegmentPool;
use super::water;

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

/// Result of a terrain cascade operation.
#[derive(Debug)]
pub struct CascadeResult {
    pub normals_updated: bool,
    pub walkability_changed: Vec<(usize, usize)>,
    pub water_cells_changed: Vec<(usize, usize)>,
    pub segments_invalidated: u16,
    pub mesh_dirty: bool,
}

/// Run the full cascade after terrain height modification.
/// Steps in order:
/// 1. Recalculate normals for cells in region
/// 2. Update walkability flags (steep slopes become unwalkable)
/// 3. Update water cells
/// 4. Invalidate pathfinding segments in region
/// 5. Mark mesh as dirty
pub fn terrain_cascade(
    heights: &[[u16; 128]; 128],
    region: &CascadeRegion,
    normals: &mut [[[f32; 3]; 128]; 128],
    walkability: &mut [[u8; 128]; 128],
    water_level: u16,
    segment_pool: &mut SegmentPool,
) -> CascadeResult {
    // Stub: returns empty result (RED phase)
    CascadeResult {
        normals_updated: false,
        walkability_changed: Vec::new(),
        water_cells_changed: Vec::new(),
        segments_invalidated: 0,
        mesh_dirty: false,
    }
}

/// Invalidate cached path segments whose src or dst falls within the modified region.
/// Uses toroidal wrapping for coordinate comparison.
/// Returns count of invalidated segments.
pub fn invalidate_segments_in_region(
    segment_pool: &mut SegmentPool,
    region: &CascadeRegion,
) -> u16 {
    // Stub: returns 0 (RED phase)
    0
}
