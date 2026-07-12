/// Validate that a building can be placed at (cell_x, cell_y).
/// Returns Ok(()) if valid, Err(reason) if invalid.
pub fn validate_placement(
    cell_x: i32,
    cell_y: i32,
    footprint_cells: &[(i32, i32)],
    walkability: &[[u8; 128]; 128],
    building_flags: &[[bool; 128]; 128],
) -> Result<(), PlacementError> {
    for &(dx, dy) in footprint_cells {
        let cx = ((cell_x + dx) & 127) as usize;
        let cy = ((cell_y + dy) & 127) as usize;
        if walkability[cy][cx] & 0x04 != 0 {
            return Err(PlacementError::Water);
        }
        if building_flags[cy][cx] {
            return Err(PlacementError::Occupied);
        }
        // Check slope (walkability bit 0x02 = unwalkable steep)
        if walkability[cy][cx] & 0x02 != 0 {
            return Err(PlacementError::TooSteep);
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementError {
    Water,
    Occupied,
    TooSteep,
    OutOfBounds,
}

/// Ghost preview state for building placement UI.
#[derive(Debug)]
pub struct GhostPreview {
    pub building_subtype: u8,
    pub cell_x: i32,
    pub cell_y: i32,
    pub rotation: u16,
    pub valid: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_walkability() -> Box<[[u8; 128]; 128]> {
        Box::new([[0u8; 128]; 128])
    }

    fn empty_building_flags() -> Box<[[bool; 128]; 128]> {
        Box::new([[false; 128]; 128])
    }

    #[test]
    fn valid_placement_on_clear_terrain() {
        let walk = empty_walkability();
        let flags = empty_building_flags();
        let footprint = vec![(0, 0), (1, 0), (0, 1), (1, 1)];
        assert_eq!(
            validate_placement(10, 10, &footprint, &walk, &flags),
            Ok(())
        );
    }

    #[test]
    fn placement_rejects_water() {
        let mut walk = empty_walkability();
        walk[10][11] = 0x04; // water at (11, 10)
        let flags = empty_building_flags();
        let footprint = vec![(0, 0), (1, 0)];
        assert_eq!(
            validate_placement(10, 10, &footprint, &walk, &flags),
            Err(PlacementError::Water)
        );
    }

    #[test]
    fn placement_rejects_occupied() {
        let walk = empty_walkability();
        let mut flags = empty_building_flags();
        flags[10][10] = true; // building at (10, 10)
        let footprint = vec![(0, 0)];
        assert_eq!(
            validate_placement(10, 10, &footprint, &walk, &flags),
            Err(PlacementError::Occupied)
        );
    }

    #[test]
    fn placement_rejects_steep() {
        let mut walk = empty_walkability();
        walk[10][10] = 0x02; // steep
        let flags = empty_building_flags();
        let footprint = vec![(0, 0)];
        assert_eq!(
            validate_placement(10, 10, &footprint, &walk, &flags),
            Err(PlacementError::TooSteep)
        );
    }

    #[test]
    fn placement_wraps_toroidally() {
        let walk = empty_walkability();
        let flags = empty_building_flags();
        // Place at edge, footprint wraps around
        let footprint = vec![(0, 0), (1, 0)]; // (127,0) and (128&127=0, 0)
        assert_eq!(
            validate_placement(127, 0, &footprint, &walk, &flags),
            Ok(())
        );
    }

    #[test]
    fn placement_water_priority_over_steep() {
        let mut walk = empty_walkability();
        walk[10][10] = 0x04 | 0x02; // both water and steep
        let flags = empty_building_flags();
        let footprint = vec![(0, 0)];
        // Water check comes first
        assert_eq!(
            validate_placement(10, 10, &footprint, &walk, &flags),
            Err(PlacementError::Water)
        );
    }

    #[test]
    fn ghost_preview_construction() {
        let ghost = GhostPreview {
            building_subtype: 1,
            cell_x: 10,
            cell_y: 20,
            rotation: 0,
            valid: true,
        };
        assert_eq!(ghost.building_subtype, 1);
        assert!(ghost.valid);
    }
}
