// Population tracking and housing capacity.
// Original binary constants from things-to-implement.md section 22.

// TODO: implement constants and functions

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
