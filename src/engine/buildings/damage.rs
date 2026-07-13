use super::state_machine::{on_destroy, transition_building_state};
use super::tick::construction_progress_target;
use super::types::*;
use crate::engine::objects::ObjectHeader;

/// Apply damage to a building. Returns true if building was destroyed.
/// Original: Building_ApplyDamage at 0x434570
pub fn apply_building_damage(
    building: &mut BuildingData,
    header: &mut ObjectHeader,
    damage: u16,
) -> bool {
    if building.state != BuildingState::Active {
        return false;
    }
    if building.damage_cooldown > 0 {
        return false;
    }

    building.damage_accumulated += damage;
    building.damage_cooldown = 4; // 4-tick cooldown between damage
    building.shake_x = 8; // visual wobble
    building.shake_z = 8;

    if header.health <= damage {
        header.health = 0;
        on_destroy(building);
        building.construction_progress = construction_progress_target(building.building_subtype);
        building.construction_phase = 4;
        transition_building_state(building, BuildingState::Destroying);
        true
    } else {
        header.health -= damage;
        false
    }
}

/// Chain damage to nearby buildings within radius.
pub fn chain_damage_radius() -> u16 {
    3 // cells
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::units::ModelType;
    use crate::engine::movement::WorldCoord;
    use crate::engine::objects::ObjectHandle;

    fn make_header(health: u16) -> ObjectHeader {
        ObjectHeader {
            model_type: ModelType::Building,
            subtype: 1,
            tribe: 0,
            state: 0,
            state_phase: 0,
            flags1: 0,
            flags2: 0,
            flags3: 0,
            object_index: ObjectHandle::new(0, 1),
            angle: 0,
            position: WorldCoord::default(),
            velocity: WorldCoord::default(),
            health,
            max_health: 600,
            next_in_cell: None,
            prev_in_cell: None,
        }
    }

    fn make_active_building() -> BuildingData {
        let mut b = BuildingData::default();
        b.state = BuildingState::Active;
        b
    }

    #[test]
    fn damage_reduces_health() {
        let mut b = make_active_building();
        let mut h = make_header(600);
        let destroyed = apply_building_damage(&mut b, &mut h, 100);
        assert!(!destroyed);
        assert_eq!(h.health, 500);
    }

    #[test]
    fn damage_sets_cooldown() {
        let mut b = make_active_building();
        let mut h = make_header(600);
        apply_building_damage(&mut b, &mut h, 50);
        assert_eq!(b.damage_cooldown, 4);
    }

    #[test]
    fn damage_sets_wobble() {
        let mut b = make_active_building();
        let mut h = make_header(600);
        apply_building_damage(&mut b, &mut h, 50);
        assert_eq!(b.shake_x, 8);
        assert_eq!(b.shake_z, 8);
    }

    #[test]
    fn damage_accumulates() {
        let mut b = make_active_building();
        let mut h = make_header(600);
        apply_building_damage(&mut b, &mut h, 50);
        assert_eq!(b.damage_accumulated, 50);
    }

    #[test]
    fn damage_destroys_at_zero_health() {
        let mut b = make_active_building();
        let mut h = make_header(100);
        let destroyed = apply_building_damage(&mut b, &mut h, 100);
        assert!(destroyed);
        assert_eq!(h.health, 0);
        assert_eq!(b.state, BuildingState::Destroying);
    }

    #[test]
    fn damage_destroys_when_exceeds_health() {
        let mut b = make_active_building();
        let mut h = make_header(50);
        let destroyed = apply_building_damage(&mut b, &mut h, 100);
        assert!(destroyed);
        assert_eq!(h.health, 0);
        assert_eq!(b.state, BuildingState::Destroying);
    }

    #[test]
    fn damage_blocked_by_cooldown() {
        let mut b = make_active_building();
        b.damage_cooldown = 2;
        let mut h = make_header(600);
        let destroyed = apply_building_damage(&mut b, &mut h, 100);
        assert!(!destroyed);
        assert_eq!(h.health, 600); // unchanged
    }

    #[test]
    fn damage_blocked_non_active_state() {
        let mut b = make_active_building();
        b.state = BuildingState::Init;
        let mut h = make_header(600);
        let destroyed = apply_building_damage(&mut b, &mut h, 100);
        assert!(!destroyed);
        assert_eq!(h.health, 600);
    }

    #[test]
    fn chain_damage_radius_is_three() {
        assert_eq!(chain_damage_radius(), 3);
    }
}
