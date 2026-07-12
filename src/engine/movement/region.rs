// Region map — 128×128 grid of terrain/connectivity cells.
// Binary base: 0x88897C
// Original: region map used by RouteTableLookup @ 0x4d7f20
//
// Each cell stores a region ID (10-bit). Units in the same region
// can walk directly to each other (Tier 1 of the 4-tier cache).
// Cross-region movement requires segment-based pathfinding.

use super::constants::*;
use super::types::{RegionMapCell, TileCoord, WorldCoord};

/// The 128×128 region map grid.
/// Populated at level load time from the level data.
pub struct RegionMap {
    cells: Vec<RegionMapCell>,
    /// Terrain flags table (binary: 0x5A3038).
    /// 16 entries indexed by terrain_class (low nibble of cell terrain_type).
    /// Bit 0x02 = walkable.
    terrain_flags: [u8; TERRAIN_FLAGS_COUNT],
}

impl RegionMap {
    /// Create an empty region map (all cells zeroed, all terrain walkable).
    pub fn new() -> Self {
        // Default: all terrain types marked walkable
        let mut terrain_flags = [0u8; TERRAIN_FLAGS_COUNT];
        for flag in terrain_flags.iter_mut() {
            *flag = TERRAIN_WALKABLE_BIT;
        }
        Self {
            cells: vec![RegionMapCell::default(); REGION_GRID_SIZE * REGION_GRID_SIZE],
            terrain_flags,
        }
    }

    /// Create a region map from raw cell data (e.g., loaded from level file).
    pub fn from_cells(cells: Vec<RegionMapCell>) -> Self {
        assert_eq!(
            cells.len(),
            REGION_GRID_SIZE * REGION_GRID_SIZE,
            "Region map must be exactly {}×{} cells",
            REGION_GRID_SIZE,
            REGION_GRID_SIZE
        );
        let mut terrain_flags = [0u8; TERRAIN_FLAGS_COUNT];
        for flag in terrain_flags.iter_mut() {
            *flag = TERRAIN_WALKABLE_BIT;
        }
        Self {
            cells,
            terrain_flags,
        }
    }

    /// Set terrain flags for a terrain class (used during level loading / testing).
    /// Binary: terrain flags table at 0x5A3038, indexed by terrain_class.
    pub fn set_terrain_flags(&mut self, terrain_class: u8, flags: u8) {
        if (terrain_class as usize) < TERRAIN_FLAGS_COUNT {
            self.terrain_flags[terrain_class as usize] = flags;
        }
    }

    /// Get terrain flags for a terrain class.
    pub fn get_terrain_flags(&self, terrain_class: u8) -> u8 {
        if (terrain_class as usize) < TERRAIN_FLAGS_COUNT {
            self.terrain_flags[terrain_class as usize]
        } else {
            0
        }
    }

    /// Get a cell by tile coordinates.
    pub fn get_cell(&self, tile: TileCoord) -> &RegionMapCell {
        &self.cells[tile.cell_index()]
    }

    /// Get a mutable cell by tile coordinates.
    pub fn get_cell_mut(&mut self, tile: TileCoord) -> &mut RegionMapCell {
        &mut self.cells[tile.cell_index()]
    }

    /// Get the region ID at the given tile.
    pub fn region_at(&self, tile: TileCoord) -> u16 {
        self.get_cell(tile).region_id()
    }

    /// Get the region ID at the given world position.
    pub fn region_at_world(&self, pos: WorldCoord) -> u16 {
        self.region_at(pos.to_tile())
    }

    /// Check if two world positions are in the same region.
    /// This is the Tier 1 check — if true, the unit can walk directly.
    /// Original: RouteTableLookup @ 0x4d7f20, step 3-4
    pub fn same_region(&self, a: WorldCoord, b: WorldCoord) -> bool {
        self.region_at_world(a) == self.region_at_world(b)
    }

    /// Check if a tile has a building on it.
    pub fn has_building(&self, tile: TileCoord) -> bool {
        self.get_cell(tile).has_building()
    }

    /// Get terrain class at a tile (low nibble of terrain_type).
    pub fn terrain_class(&self, tile: TileCoord) -> u8 {
        self.get_cell(tile).terrain_class()
    }

    /// Check if a tile is walkable.
    /// Reads terrain_class from the cell, then checks the terrain flags table.
    /// Original: terrain flags at 0x5A3038, bit 0x02 = walkable.
    pub fn is_walkable(&self, tile: TileCoord) -> bool {
        let tc = self.terrain_class(tile);
        self.terrain_flags[tc as usize] & TERRAIN_WALKABLE_BIT != 0
    }

    /// Check if a world position is on walkable terrain.
    pub fn is_walkable_world(&self, pos: WorldCoord) -> bool {
        self.is_walkable(pos.to_tile())
    }

    /// Set region data for a cell (used during level loading / testing).
    pub fn set_cell_region(&mut self, tile: TileCoord, region_id: u16) {
        let cell = self.get_cell_mut(tile);
        // Preserve upper 6 bits, set lower 10
        cell.region_id_raw = (cell.region_id_raw & !REGION_ID_MASK) | (region_id & REGION_ID_MASK);
    }
}

impl Default for RegionMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate spiral neighbor offsets around a center point.
/// Yields (dx, dz) pairs in concentric rings, up to `max_neighbors` total.
/// Original: FUN_0041cc90 / FUN_0041cd90 spiral search pattern.
///
/// The pattern expands: (0,1), (1,0), (0,-1), (-1,0), (1,1), (1,-1), ...
/// Each ring has 8*ring cells. The offsets are tile-granularity (multiply by 2
/// for tile_coord deltas since tiles step by 2).
pub fn spiral_neighbors(max_neighbors: usize) -> Vec<(i8, i8)> {
    let mut offsets = Vec::with_capacity(max_neighbors);

    // Ring 1: 4 cardinal neighbors
    // Ring 2: 8 neighbors (cardinal + diagonal at distance 2)
    // etc.
    let mut ring = 1i8;
    while offsets.len() < max_neighbors {
        // Walk the perimeter of the current ring
        // Top edge: (-ring..=ring, -ring)
        for dx in -ring..=ring {
            if offsets.len() >= max_neighbors {
                break;
            }
            offsets.push((dx, -ring));
        }
        // Right edge: (ring, -ring+1..=ring)
        for dz in (-ring + 1)..=ring {
            if offsets.len() >= max_neighbors {
                break;
            }
            offsets.push((ring, dz));
        }
        // Bottom edge: (ring-1..=-ring, ring)
        for dx in (-ring..ring).rev() {
            if offsets.len() >= max_neighbors {
                break;
            }
            offsets.push((dx, ring));
        }
        // Left edge: (-ring, ring-1..=-ring+1)
        for dz in ((-ring + 1)..ring).rev() {
            if offsets.len() >= max_neighbors {
                break;
            }
            offsets.push((-ring, dz));
        }
        ring += 1;
        if ring > 16 {
            break; // Safety limit
        }
    }

    offsets.truncate(max_neighbors);
    offsets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_map_all_same_region() {
        let map = RegionMap::new();
        // All cells start as region 0
        assert!(map.same_region(
            WorldCoord::new(0x0500, 0x0500),
            WorldCoord::new(0x2000, 0x3000)
        ));
    }

    #[test]
    fn different_regions_detected() {
        let mut map = RegionMap::new();
        let tile_a = TileCoord::new(0x04, 0x04);
        let tile_b = TileCoord::new(0x10, 0x10);

        map.set_cell_region(tile_a, 1);
        map.set_cell_region(tile_b, 2);

        assert!(!map.same_region(tile_a.to_world(), tile_b.to_world()));
    }

    #[test]
    fn same_region_detected() {
        let mut map = RegionMap::new();
        let tile_a = TileCoord::new(0x04, 0x04);
        let tile_b = TileCoord::new(0x10, 0x10);

        map.set_cell_region(tile_a, 5);
        map.set_cell_region(tile_b, 5);

        assert!(map.same_region(tile_a.to_world(), tile_b.to_world()));
    }

    #[test]
    fn region_id_10bit_mask() {
        let mut map = RegionMap::new();
        let tile = TileCoord::new(0x02, 0x02);
        // Set region 1023 (max 10-bit value)
        map.set_cell_region(tile, 1023);
        assert_eq!(map.region_at(tile), 1023);
        // Try overflowing — should clamp to 10 bits
        map.set_cell_region(tile, 0xFFFF);
        assert_eq!(map.region_at(tile), 1023);
    }

    #[test]
    fn building_flag() {
        let mut map = RegionMap::new();
        let tile = TileCoord::new(0x06, 0x08);
        assert!(!map.has_building(tile));
        map.get_cell_mut(tile).flags_high = CELL_HAS_BUILDING;
        assert!(map.has_building(tile));
    }

    #[test]
    fn default_terrain_walkable() {
        let map = RegionMap::new();
        // All terrain classes default to walkable
        let tile = TileCoord::new(0x04, 0x04);
        assert!(map.is_walkable(tile));
    }

    #[test]
    fn unwalkable_terrain() {
        let mut map = RegionMap::new();
        let tile = TileCoord::new(0x04, 0x04);

        // Set cell terrain_type to class 3 (water)
        map.get_cell_mut(tile).terrain_type = 3;
        // Mark terrain class 3 as unwalkable (clear walkable bit)
        map.set_terrain_flags(3, 0x00);

        assert!(!map.is_walkable(tile));
    }

    #[test]
    fn spiral_neighbors_count() {
        let offsets = super::spiral_neighbors(32);
        assert_eq!(offsets.len(), 32);
    }

    #[test]
    fn spiral_neighbors_no_center() {
        let offsets = super::spiral_neighbors(32);
        // (0, 0) should NOT be in the spiral — it starts from ring 1
        assert!(!offsets.contains(&(0, 0)));
    }

    #[test]
    fn spiral_neighbors_first_ring() {
        let offsets = super::spiral_neighbors(32);
        // First ring should contain all 8 neighbors at distance 1
        let ring1: Vec<_> = offsets
            .iter()
            .filter(|(dx, dz)| dx.abs() <= 1 && dz.abs() <= 1)
            .collect();
        assert_eq!(ring1.len(), 8);
    }
}
