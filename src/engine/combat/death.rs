use crate::engine::objects::handle::ObjectHandle;

/// Actions required to process a unit death.
pub struct DeathActions {
    pub handle: ObjectHandle,
    pub tribe: u8,
    pub last_attacker_tribe: Option<u8>,
}

/// Determine cleanup actions for a dead unit.
/// Caller must:
/// 1. Remove from cell grid
/// 2. Destroy from object pool
/// 3. Decrement tribe population
/// 4. If last_attacker_tribe is Some, increment that tribe's kill count
pub fn process_death(
    handle: ObjectHandle,
    tribe: u8,
    last_attacker_tribe: Option<u8>,
) -> DeathActions {
    DeathActions {
        handle,
        tribe,
        last_attacker_tribe,
    }
}

/// Drum tower auto-attack range (world coordinate units).
/// About 6 cells (128 world units per cell).
pub const DRUM_TOWER_RANGE: u32 = 768;

/// Drum tower subtype value.
pub const DRUM_TOWER_SUBTYPE: u8 = 4;

/// Check if a drum tower should fire at a target.
/// Returns true if target is within range and is an enemy.
pub fn should_drum_tower_fire(tower_tribe: u8, target_tribe: u8, distance_sq: u64) -> bool {
    tower_tribe != target_tribe
        && distance_sq <= (DRUM_TOWER_RANGE as u64 * DRUM_TOWER_RANGE as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn h(slot: u16) -> ObjectHandle {
        ObjectHandle::new(slot, 1)
    }

    #[test]
    fn process_death_returns_correct_actions() {
        let actions = process_death(h(42), 1, Some(2));
        assert_eq!(actions.handle, h(42));
        assert_eq!(actions.tribe, 1);
        assert_eq!(actions.last_attacker_tribe, Some(2));
    }

    #[test]
    fn process_death_no_attacker() {
        let actions = process_death(h(10), 0, None);
        assert_eq!(actions.handle, h(10));
        assert_eq!(actions.tribe, 0);
        assert!(actions.last_attacker_tribe.is_none());
    }

    #[test]
    fn should_drum_tower_fire_same_tribe_returns_false() {
        assert!(!should_drum_tower_fire(1, 1, 100));
    }

    #[test]
    fn should_drum_tower_fire_beyond_range_returns_false() {
        let beyond = (DRUM_TOWER_RANGE as u64 + 1) * (DRUM_TOWER_RANGE as u64 + 1);
        assert!(!should_drum_tower_fire(0, 1, beyond));
    }

    #[test]
    fn should_drum_tower_fire_enemy_in_range_returns_true() {
        let in_range = (DRUM_TOWER_RANGE as u64 - 1) * (DRUM_TOWER_RANGE as u64 - 1);
        assert!(should_drum_tower_fire(0, 1, in_range));
    }

    #[test]
    fn should_drum_tower_fire_at_exact_range() {
        let exact = DRUM_TOWER_RANGE as u64 * DRUM_TOWER_RANGE as u64;
        assert!(should_drum_tower_fire(0, 1, exact));
    }

    #[test]
    fn drum_tower_range_is_768() {
        assert_eq!(DRUM_TOWER_RANGE, 768);
    }
}
