#[cfg(test)]
mod tests {
    use crate::engine::movement::segment::SegmentPool;
    use crate::engine::movement::types::TileCoord;
    use crate::engine::terrain::cascade::*;
    use crate::engine::terrain::modify::*;
    use crate::engine::terrain::water::*;

    // ============ modify_height tests ============

    #[test]
    fn terrain_modify_height_raises_toward_target() {
        let mut heights = [[0u16; 128]; 128];
        heights[10][10] = 100;
        let changed = modify_height(&mut heights, 10, 10, 200, 50);
        assert!(changed);
        assert_eq!(heights[10][10], 150);
    }

    #[test]
    fn terrain_modify_height_lowers_toward_target() {
        let mut heights = [[0u16; 128]; 128];
        heights[5][5] = 500;
        let changed = modify_height(&mut heights, 5, 5, 300, 100);
        assert!(changed);
        assert_eq!(heights[5][5], 400);
    }

    #[test]
    fn terrain_modify_height_clamps_at_target() {
        let mut heights = [[0u16; 128]; 128];
        heights[3][3] = 90;
        // Rate exceeds remaining distance
        let changed = modify_height(&mut heights, 3, 3, 100, 50);
        assert!(changed);
        assert_eq!(heights[3][3], 100); // Should clamp to target, not overshoot
    }

    #[test]
    fn terrain_modify_height_no_change_at_target() {
        let mut heights = [[0u16; 128]; 128];
        heights[7][7] = 200;
        let changed = modify_height(&mut heights, 7, 7, 200, 50);
        assert!(!changed);
        assert_eq!(heights[7][7], 200);
    }

    #[test]
    fn terrain_modify_height_wraps_coordinates() {
        let mut heights = [[0u16; 128]; 128];
        // x=130 should wrap to x=2, y=256 should wrap to y=0
        heights[0][2] = 100;
        let changed = modify_height(&mut heights, 130, 256, 200, 50);
        assert!(changed);
        assert_eq!(heights[0][2], 150);
    }

    // ============ modify_height_area tests ============

    #[test]
    fn terrain_modify_height_area_raises_region() {
        let mut heights = [[0u16; 128]; 128];
        let changed = modify_height_area(&mut heights, 64, 64, 1, 500, 500);
        assert!(changed);
        // Center and surrounding cells within radius 1 should be at target
        assert_eq!(heights[64][64], 500);
        assert_eq!(heights[63][64], 500);
        assert_eq!(heights[65][64], 500);
        assert_eq!(heights[64][63], 500);
        assert_eq!(heights[64][65], 500);
    }

    #[test]
    fn terrain_modify_height_area_no_change_already_at_target() {
        let mut heights = [[500u16; 128]; 128];
        let changed = modify_height_area(&mut heights, 64, 64, 2, 500, 100);
        assert!(!changed);
    }

    // ============ flatten_area tests ============

    #[test]
    fn terrain_flatten_area_averages_heights() {
        let mut heights = [[0u16; 128]; 128];
        // Set a 3x3 area with varying heights around (10,10)
        heights[9][9] = 100;
        heights[9][10] = 200;
        heights[9][11] = 100;
        heights[10][9] = 200;
        heights[10][10] = 300;
        heights[10][11] = 200;
        heights[11][9] = 100;
        heights[11][10] = 200;
        heights[11][11] = 100;
        // Average = (100+200+100+200+300+200+100+200+100)/9 = 1500/9 = 166

        let changed = flatten_area(&mut heights, 10, 10, 1);
        assert!(changed);
        // All cells in area should move toward the average
        // With large enough rate, they should reach the average
    }

    // ============ is_water_cell tests ============

    #[test]
    fn terrain_is_water_cell_below_level() {
        assert!(is_water_cell(50, 100));
    }

    #[test]
    fn terrain_is_water_cell_at_level() {
        assert!(!is_water_cell(100, 100));
    }

    #[test]
    fn terrain_is_water_cell_above_level() {
        assert!(!is_water_cell(200, 100));
    }

    // ============ update_water_cells tests ============

    #[test]
    fn terrain_update_water_cells_marks_low_cells() {
        let mut heights = [[500u16; 128]; 128];
        let mut water_flags = [[0u8; 128]; 128];
        let region = CascadeRegion {
            min_x: 0,
            min_y: 0,
            max_x: 3,
            max_y: 3,
        };

        // Set some cells below water level
        heights[0][0] = 10;
        heights[1][1] = 10;

        let changed = update_water_cells(&heights, &mut water_flags, &region, 100);
        assert!(changed.len() >= 2);
        assert_ne!(water_flags[0][0] & WATER_WALKABILITY_FLAG, 0);
        assert_ne!(water_flags[1][1] & WATER_WALKABILITY_FLAG, 0);
        // Cell above water level should not have flag
        assert_eq!(water_flags[2][2] & WATER_WALKABILITY_FLAG, 0);
    }

    #[test]
    fn terrain_update_water_cells_clears_raised_cells() {
        let mut heights = [[500u16; 128]; 128];
        let mut water_flags = [[0u8; 128]; 128];
        let region = CascadeRegion {
            min_x: 0,
            min_y: 0,
            max_x: 3,
            max_y: 3,
        };

        // Pre-set water flag on a cell that's now above water level
        water_flags[2][2] = WATER_WALKABILITY_FLAG;

        let changed = update_water_cells(&heights, &mut water_flags, &region, 100);
        assert!(changed.contains(&(2, 2)));
        assert_eq!(water_flags[2][2] & WATER_WALKABILITY_FLAG, 0);
    }

    // ============ invalidate_segments_in_region tests ============

    #[test]
    fn terrain_invalidate_segments_in_region_drops_matching() {
        let mut pool = SegmentPool::new();

        // Create a segment with src in the region
        let idx = pool.allocate().unwrap();
        pool.add_ref(idx);
        let seg = pool.get_mut(idx).unwrap();
        seg.start_tile_x = 10;
        seg.start_tile_z = 10;
        seg.end_tile_x = 50;
        seg.end_tile_z = 50;

        let region = CascadeRegion {
            min_x: 5,
            min_y: 5,
            max_x: 15,
            max_y: 15,
        };
        let count = invalidate_segments_in_region(&mut pool, &region);
        assert_eq!(count, 1);
        assert_eq!(pool.get(idx).unwrap().ref_count, 0);
    }

    #[test]
    fn terrain_invalidate_segments_in_region_keeps_outside() {
        let mut pool = SegmentPool::new();

        // Create a segment entirely outside the region
        let idx = pool.allocate().unwrap();
        pool.add_ref(idx);
        let seg = pool.get_mut(idx).unwrap();
        seg.start_tile_x = 100;
        seg.start_tile_z = 100;
        seg.end_tile_x = 110;
        seg.end_tile_z = 110;

        let region = CascadeRegion {
            min_x: 0,
            min_y: 0,
            max_x: 10,
            max_y: 10,
        };
        let count = invalidate_segments_in_region(&mut pool, &region);
        assert_eq!(count, 0);
        assert!(pool.get(idx).unwrap().ref_count > 0);
    }

    #[test]
    fn terrain_invalidate_segments_dst_in_region() {
        let mut pool = SegmentPool::new();

        // Create a segment with dst in the region (src outside)
        let idx = pool.allocate().unwrap();
        pool.add_ref(idx);
        let seg = pool.get_mut(idx).unwrap();
        seg.start_tile_x = 100;
        seg.start_tile_z = 100;
        seg.end_tile_x = 10;
        seg.end_tile_z = 10;

        let region = CascadeRegion {
            min_x: 5,
            min_y: 5,
            max_x: 15,
            max_y: 15,
        };
        let count = invalidate_segments_in_region(&mut pool, &region);
        assert_eq!(count, 1);
    }

    // ============ terrain_cascade tests ============

    #[test]
    fn terrain_cascade_updates_normals() {
        let mut heights = [[100u16; 128]; 128];
        let mut normals = [[[0.0f32; 3]; 128]; 128];
        let mut walkability = [[0u8; 128]; 128];
        let mut pool = SegmentPool::new();
        let region = CascadeRegion {
            min_x: 10,
            min_y: 10,
            max_x: 12,
            max_y: 12,
        };

        let result = terrain_cascade(
            &heights,
            &region,
            &mut normals,
            &mut walkability,
            50,
            &mut pool,
        );
        assert!(result.normals_updated);
        assert!(result.mesh_dirty);
    }

    #[test]
    fn terrain_cascade_detects_steep_slopes() {
        let mut heights = [[100u16; 128]; 128];
        // Create a steep slope (difference > 512)
        heights[10][10] = 100;
        heights[10][11] = 700; // diff = 600 > 512

        let mut normals = [[[0.0f32; 3]; 128]; 128];
        let mut walkability = [[0u8; 128]; 128];
        let mut pool = SegmentPool::new();
        let region = CascadeRegion {
            min_x: 9,
            min_y: 9,
            max_x: 12,
            max_y: 12,
        };

        let result = terrain_cascade(
            &heights,
            &region,
            &mut normals,
            &mut walkability,
            50,
            &mut pool,
        );
        // Cell (10, 10) should be marked as unwalkable due to steep slope
        // The steep slope flag is 0x02
        assert_ne!(
            walkability[10][10] & 0x02,
            0,
            "Cell with steep slope should be marked unwalkable"
        );
    }

    #[test]
    fn terrain_cascade_updates_water() {
        let mut heights = [[500u16; 128]; 128];
        heights[10][10] = 10; // Below water level

        let mut normals = [[[0.0f32; 3]; 128]; 128];
        let mut walkability = [[0u8; 128]; 128];
        let mut pool = SegmentPool::new();
        let region = CascadeRegion {
            min_x: 9,
            min_y: 9,
            max_x: 12,
            max_y: 12,
        };

        let result = terrain_cascade(
            &heights,
            &region,
            &mut normals,
            &mut walkability,
            100,
            &mut pool,
        );
        assert!(!result.water_cells_changed.is_empty());
        assert_ne!(walkability[10][10] & WATER_WALKABILITY_FLAG, 0);
    }

    #[test]
    fn terrain_cascade_invalidates_segments() {
        let mut heights = [[100u16; 128]; 128];
        let mut normals = [[[0.0f32; 3]; 128]; 128];
        let mut walkability = [[0u8; 128]; 128];
        let mut pool = SegmentPool::new();

        // Create a segment in the region
        let idx = pool.allocate().unwrap();
        pool.add_ref(idx);
        let seg = pool.get_mut(idx).unwrap();
        seg.start_tile_x = 10;
        seg.start_tile_z = 10;
        seg.end_tile_x = 50;
        seg.end_tile_z = 50;

        let region = CascadeRegion {
            min_x: 5,
            min_y: 5,
            max_x: 15,
            max_y: 15,
        };
        let result = terrain_cascade(
            &heights,
            &region,
            &mut normals,
            &mut walkability,
            50,
            &mut pool,
        );
        assert_eq!(result.segments_invalidated, 1);
    }

    // ============ Toroidal wrapping tests ============

    #[test]
    fn terrain_modify_height_area_wraps_toroidally() {
        let mut heights = [[0u16; 128]; 128];
        // Place center at edge, radius should wrap around
        let changed = modify_height_area(&mut heights, 0, 0, 1, 500, 500);
        assert!(changed);
        assert_eq!(heights[0][0], 500);
        assert_eq!(heights[127][0], 500); // Wrapped y-1
        assert_eq!(heights[0][127], 500); // Wrapped x-1
    }

    #[test]
    fn terrain_normal_calculation_flat() {
        // For a flat surface, normal should point straight up (0, 1, 0)
        let heights = [[100u16; 128]; 128];
        let mut normals = [[[0.0f32; 3]; 128]; 128];
        let mut walkability = [[0u8; 128]; 128];
        let mut pool = SegmentPool::new();
        let region = CascadeRegion {
            min_x: 10,
            min_y: 10,
            max_x: 11,
            max_y: 11,
        };

        terrain_cascade(
            &heights,
            &region,
            &mut normals,
            &mut walkability,
            50,
            &mut pool,
        );

        let n = normals[10][10];
        assert!(
            (n[0]).abs() < 0.01,
            "x component should be ~0 for flat terrain, got {}",
            n[0]
        );
        assert!(
            n[1] > 0.99,
            "y component should be ~1 for flat terrain, got {}",
            n[1]
        );
        assert!(
            (n[2]).abs() < 0.01,
            "z component should be ~0 for flat terrain, got {}",
            n[2]
        );
    }
}
