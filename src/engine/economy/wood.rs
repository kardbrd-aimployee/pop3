// Wood storage constants and tracking.
// Construction and training wood costs.
// Tree and building search via CellGrid.

use crate::data::units::ModelType;
use crate::engine::buildings::{BuildingData, BuildingState};
use crate::engine::movement::WorldCoord;
use crate::engine::objects::cell_grid::CellGrid;
use crate::engine::objects::pool::ObjectPool;
use crate::engine::objects::types::{GameObjectData, PoolSlot};

/// Maximum tree subtype value (0-8 are tree variants in original scenery table).
const MAX_TREE_SUBTYPE: u8 = 8;

/// Find the nearest tree to a position by querying the CellGrid for scenery objects.
/// Trees are scenery objects (ModelType::Scenery) with subtypes 0-8 (tree variants).
/// Searches in expanding rings around the unit's tile position up to max_radius.
pub fn find_nearest_tree_position(
    unit_pos: &WorldCoord,
    cell_grid: &CellGrid,
    pool: &ObjectPool,
) -> Option<WorldCoord> {
    let tile = unit_pos.to_tile();
    let max_radius: i32 = 15;

    for radius in 1..=max_radius {
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                if dx.abs() != radius && dz.abs() != radius {
                    continue; // ring only
                }
                let cx = ((tile.x as i32 + dx) & 127) as u8;
                let cz = ((tile.z as i32 + dz) & 127) as u8;
                let cell_idx = cz as usize * 128 + cx as usize;

                // Walk linked list in this cell
                let mut current = cell_grid.cell_head(cell_idx);
                while let Some(handle) = current {
                    if let Some(obj) = pool.get(handle) {
                        if obj.header.model_type == ModelType::Scenery
                            && obj.header.subtype <= MAX_TREE_SUBTYPE
                        {
                            return Some(obj.header.position);
                        }
                        current = obj.header.next_in_cell;
                    } else {
                        break;
                    }
                }
            }
        }
    }
    None
}

/// Find nearest active building owned by tribe to deposit wood at.
pub fn find_nearest_building_position(
    unit_pos: &WorldCoord,
    tribe: u8,
    cell_grid: &CellGrid,
    pool: &ObjectPool,
) -> Option<WorldCoord> {
    let tile = unit_pos.to_tile();
    let max_radius: i32 = 20;

    for radius in 1..=max_radius {
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                if dx.abs() != radius && dz.abs() != radius {
                    continue;
                }
                let cx = ((tile.x as i32 + dx) & 127) as u8;
                let cz = ((tile.z as i32 + dz) & 127) as u8;
                let cell_idx = cz as usize * 128 + cx as usize;

                let mut current = cell_grid.cell_head(cell_idx);
                while let Some(handle) = current {
                    if let Some(obj) = pool.get(handle) {
                        if let GameObjectData::Building(ref bd) = obj.data {
                            if obj.header.tribe == tribe
                                && bd.state == BuildingState::Active
                            {
                                return Some(obj.header.position);
                            }
                        }
                        current = obj.header.next_in_cell;
                    } else {
                        break;
                    }
                }
            }
        }
    }
    None
}

/// Construction wood costs by building subtype.
pub const WOOD_COST_SMALL_HUT: u16 = 3;
pub const WOOD_COST_MEDIUM_HUT: u16 = 5;
pub const WOOD_COST_LARGE_HUT: u16 = 7;
pub const WOOD_COST_DRUM_TOWER: u16 = 5;
pub const WOOD_COST_TEMPLE: u16 = 6;
pub const WOOD_COST_TRAINING: u16 = 5;
pub const WOOD_COST_DEFAULT: u16 = 4;

/// Training wood costs by target person subtype.
pub const WOOD_TRAIN_WARRIOR: u16 = 3;
pub const WOOD_TRAIN_SPY: u16 = 2;
pub const WOOD_TRAIN_PREACHER: u16 = 2;
pub const WOOD_TRAIN_SUPER_WARRIOR: u16 = 5;

/// Wood cost to construct a building of given subtype.
pub fn construction_wood_cost(building_subtype: u8) -> u16 {
    match building_subtype {
        1 => WOOD_COST_SMALL_HUT,
        2 => WOOD_COST_MEDIUM_HUT,
        3 => WOOD_COST_LARGE_HUT,
        4 => WOOD_COST_DRUM_TOWER,
        5 => WOOD_COST_TEMPLE,
        6 | 7 | 8 => WOOD_COST_TRAINING,
        _ => WOOD_COST_DEFAULT,
    }
}

/// Wood cost to train a unit of given target subtype.
pub fn training_wood_cost(target_subtype: u8) -> u16 {
    match target_subtype {
        3 => WOOD_TRAIN_WARRIOR,       // Warrior
        4 => WOOD_TRAIN_PREACHER,      // Religious/Preacher
        5 => WOOD_TRAIN_SPY,           // Spy
        6 => WOOD_TRAIN_SUPER_WARRIOR, // Super Warrior
        _ => 0,
    }
}

/// Sum total wood stored across a slice of wood values.
pub fn total_wood_stored(wood_values: &[u16]) -> u32 {
    wood_values.iter().map(|&w| w as u32).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Construction cost tests --

    #[test]
    fn test_construction_wood_cost_small_hut() {
        assert_eq!(construction_wood_cost(1), 3);
    }

    #[test]
    fn test_construction_wood_cost_medium_hut() {
        assert_eq!(construction_wood_cost(2), 5);
    }

    #[test]
    fn test_construction_wood_cost_large_hut() {
        assert_eq!(construction_wood_cost(3), 7);
    }

    #[test]
    fn test_construction_wood_cost_drum_tower() {
        assert_eq!(construction_wood_cost(4), 5);
    }

    #[test]
    fn test_construction_wood_cost_temple() {
        assert_eq!(construction_wood_cost(5), 6);
    }

    #[test]
    fn test_construction_wood_cost_training() {
        assert_eq!(construction_wood_cost(6), 5);
        assert_eq!(construction_wood_cost(7), 5);
        assert_eq!(construction_wood_cost(8), 5);
    }

    #[test]
    fn test_construction_wood_cost_default() {
        assert_eq!(construction_wood_cost(0), 4);
        assert_eq!(construction_wood_cost(99), 4);
    }

    // -- Training cost tests --

    #[test]
    fn test_training_wood_cost_warrior() {
        assert_eq!(training_wood_cost(3), 3);
    }

    #[test]
    fn test_training_wood_cost_preacher() {
        assert_eq!(training_wood_cost(4), 2);
    }

    #[test]
    fn test_training_wood_cost_spy() {
        assert_eq!(training_wood_cost(5), 2);
    }

    #[test]
    fn test_training_wood_cost_super_warrior() {
        assert_eq!(training_wood_cost(6), 5);
    }

    #[test]
    fn test_training_wood_cost_unknown() {
        assert_eq!(training_wood_cost(0), 0);
        assert_eq!(training_wood_cost(1), 0);
        assert_eq!(training_wood_cost(2), 0);
    }

    // -- Wood storage tests --

    #[test]
    fn test_total_wood_stored_empty() {
        assert_eq!(total_wood_stored(&[]), 0);
    }

    #[test]
    fn test_total_wood_stored_single() {
        assert_eq!(total_wood_stored(&[10]), 10);
    }

    #[test]
    fn test_total_wood_stored_multiple() {
        assert_eq!(total_wood_stored(&[5, 10, 3, 7]), 25);
    }
}
