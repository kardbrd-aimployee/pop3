// Wood storage constants and tracking.
// Construction and training wood costs.

// TODO: implement constants and functions

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
