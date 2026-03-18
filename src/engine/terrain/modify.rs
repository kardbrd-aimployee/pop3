// Terrain height modification functions.
// Implements gradual height changes per tick, matching Terrain_ModifyHeight at 0x4ea2e0.

/// Modify a single cell's height toward a target value, moving by at most `rate` per call.
/// Coordinates wrap toroidally (& 127). Returns true if height changed.
pub fn modify_height(
    heights: &mut [[u16; 128]; 128],
    x: usize,
    y: usize,
    target: u16,
    rate: u16,
) -> bool {
    // Stub: returns false (RED phase)
    false
}

/// Apply modify_height to all cells within `radius` of (cx, cy), using toroidal distance.
/// Returns true if any height changed.
pub fn modify_height_area(
    heights: &mut [[u16; 128]; 128],
    cx: usize,
    cy: usize,
    radius: usize,
    target: u16,
    rate: u16,
) -> bool {
    // Stub: returns false (RED phase)
    false
}

/// Compute average height in area and modify all cells toward that average.
/// Returns true if any height changed.
pub fn flatten_area(
    heights: &mut [[u16; 128]; 128],
    cx: usize,
    cy: usize,
    radius: usize,
) -> bool {
    // Stub: returns false (RED phase)
    false
}
