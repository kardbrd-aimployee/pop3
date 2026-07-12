// Population tracking and housing capacity.
// Original binary constants are documented in docs/specs/person_units.md (Appendix O).

/// Maximum occupants per hut level.
pub const MAX_POP_VALUE_HUT_1: u16 = 3;
pub const MAX_POP_VALUE_HUT_2: u16 = 4;
pub const MAX_POP_VALUE_HUT_3: u16 = 5;

/// Tribe population cap (absolute maximum regardless of housing).
pub const MAX_POP_VALUE: u16 = 199;

/// Returns max occupants for a hut level.
pub fn hut_capacity(level: u8) -> u16 {
    match level {
        1 => MAX_POP_VALUE_HUT_1,
        2 => MAX_POP_VALUE_HUT_2,
        3 => MAX_POP_VALUE_HUT_3,
        _ => 0,
    }
}

/// Calculate total housing capacity from hut counts by level.
/// hut_counts: [level_1_count, level_2_count, level_3_count]
pub fn calculate_housing_capacity(hut_counts: [u16; 3]) -> u16 {
    let raw = hut_counts[0] * MAX_POP_VALUE_HUT_1
        + hut_counts[1] * MAX_POP_VALUE_HUT_2
        + hut_counts[2] * MAX_POP_VALUE_HUT_3;
    raw.min(MAX_POP_VALUE)
}

/// Check if tribe can spawn more units.
pub fn can_spawn(current_population: u16, max_population: u16) -> bool {
    current_population < max_population
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_pop_value() {
        assert_eq!(MAX_POP_VALUE, 199);
    }

    #[test]
    fn test_hut_capacity_levels() {
        assert_eq!(hut_capacity(1), 3);
        assert_eq!(hut_capacity(2), 4);
        assert_eq!(hut_capacity(3), 5);
        assert_eq!(hut_capacity(0), 0);
        assert_eq!(hut_capacity(4), 0);
    }

    #[test]
    fn test_calculate_housing_capacity_basic() {
        // 10 level-1 huts = 30 capacity
        assert_eq!(calculate_housing_capacity([10, 0, 0]), 30);
    }

    #[test]
    fn test_calculate_housing_capacity_mixed() {
        // 5 level-1 (15) + 3 level-2 (12) + 2 level-3 (10) = 37
        assert_eq!(calculate_housing_capacity([5, 3, 2]), 37);
    }

    #[test]
    fn test_calculate_housing_capacity_capped() {
        // 100 level-3 huts = 500, but capped at 199
        assert_eq!(calculate_housing_capacity([0, 0, 100]), 199);
    }

    #[test]
    fn test_calculate_housing_capacity_zero() {
        assert_eq!(calculate_housing_capacity([0, 0, 0]), 0);
    }

    #[test]
    fn test_can_spawn_below_cap() {
        assert!(can_spawn(10, 50));
    }

    #[test]
    fn test_can_spawn_at_cap() {
        assert!(!can_spawn(50, 50));
    }

    #[test]
    fn test_can_spawn_over_cap() {
        assert!(!can_spawn(51, 50));
    }
}
