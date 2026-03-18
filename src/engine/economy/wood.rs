// Wood storage constants and tracking.
// Construction and training wood costs.

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
