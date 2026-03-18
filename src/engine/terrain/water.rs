// Water/land transition logic for terrain modification cascade.
// Cells transition between water and land based on height vs water level.

use super::cascade::CascadeRegion;

const GRID_MASK: usize = 127;

/// Walkability flag indicating a cell is water.
pub const WATER_WALKABILITY_FLAG: u8 = 0x04;

/// Check if a cell is water based on height vs water level.
pub fn is_water_cell(height: u16, water_level: u16) -> bool {
    height < water_level
}

/// Update water flags for cells in the given region.
/// For each cell in region, sets WATER_WALKABILITY_FLAG if height < water_level,
/// clears it otherwise. Returns list of cells whose water status changed.
pub fn update_water_cells(
    heights: &[[u16; 128]; 128],
    water_flags: &mut [[u8; 128]; 128],
    region: &CascadeRegion,
    water_level: u16,
) -> Vec<(usize, usize)> {
    let mut changed = Vec::new();

    region.for_each_cell(|x, y| {
        let is_water = is_water_cell(heights[y][x], water_level);
        let was_water = water_flags[y][x] & WATER_WALKABILITY_FLAG != 0;

        if is_water && !was_water {
            water_flags[y][x] |= WATER_WALKABILITY_FLAG;
            changed.push((x, y));
        } else if !is_water && was_water {
            water_flags[y][x] &= !WATER_WALKABILITY_FLAG;
            changed.push((x, y));
        }
    });

    changed
}
