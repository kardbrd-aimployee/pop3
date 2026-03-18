// Terrain height modification functions.
// Implements gradual height changes per tick, matching Terrain_ModifyHeight at 0x4ea2e0.

const GRID_SIZE: usize = 128;
const GRID_MASK: usize = 127;

/// Modify a single cell's height toward a target value, moving by at most `rate` per call.
/// Coordinates wrap toroidally (& 127). Returns true if height changed.
pub fn modify_height(
    heights: &mut [[u16; GRID_SIZE]; GRID_SIZE],
    x: usize,
    y: usize,
    target: u16,
    rate: u16,
) -> bool {
    let wx = x & GRID_MASK;
    let wy = y & GRID_MASK;
    let current = heights[wy][wx];

    if current == target {
        return false;
    }

    let new_height = if current < target {
        // Raising: move up by rate, clamped to target
        current.saturating_add(rate).min(target)
    } else {
        // Lowering: move down by rate, clamped to target
        current.saturating_sub(rate).max(target)
    };

    heights[wy][wx] = new_height;
    true
}

/// Compute toroidal distance between two coordinates on the 128-grid.
fn toroidal_dist(a: usize, b: usize) -> usize {
    let a = a & GRID_MASK;
    let b = b & GRID_MASK;
    let d = if a > b { a - b } else { b - a };
    d.min(GRID_SIZE - d)
}

/// Apply modify_height to all cells within `radius` of (cx, cy), using toroidal distance.
/// Returns true if any height changed.
pub fn modify_height_area(
    heights: &mut [[u16; GRID_SIZE]; GRID_SIZE],
    cx: usize,
    cy: usize,
    radius: usize,
    target: u16,
    rate: u16,
) -> bool {
    let mut any_changed = false;
    let r = radius as isize;

    for dy in -r..=r {
        for dx in -r..=r {
            let x = ((cx as isize + dx).rem_euclid(GRID_SIZE as isize)) as usize;
            let y = ((cy as isize + dy).rem_euclid(GRID_SIZE as isize)) as usize;

            // Check toroidal distance (Chebyshev / square radius)
            if toroidal_dist(cx, x) <= radius && toroidal_dist(cy, y) <= radius {
                if modify_height(heights, x, y, target, rate) {
                    any_changed = true;
                }
            }
        }
    }

    any_changed
}

/// Compute average height in area and modify all cells toward that average.
/// Returns true if any height changed.
pub fn flatten_area(
    heights: &mut [[u16; GRID_SIZE]; GRID_SIZE],
    cx: usize,
    cy: usize,
    radius: usize,
) -> bool {
    // First pass: compute average height
    let mut sum: u64 = 0;
    let mut count: u64 = 0;
    let r = radius as isize;

    for dy in -r..=r {
        for dx in -r..=r {
            let x = ((cx as isize + dx).rem_euclid(GRID_SIZE as isize)) as usize;
            let y = ((cy as isize + dy).rem_euclid(GRID_SIZE as isize)) as usize;

            if toroidal_dist(cx, x) <= radius && toroidal_dist(cy, y) <= radius {
                sum += heights[y][x] as u64;
                count += 1;
            }
        }
    }

    if count == 0 {
        return false;
    }

    let avg = (sum / count) as u16;

    // Second pass: modify toward average (use large rate to flatten in one call)
    modify_height_area(heights, cx, cy, radius, avg, u16::MAX)
}
