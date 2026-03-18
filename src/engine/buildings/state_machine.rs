use super::types::*;

/// Validates and applies a state transition on a building.
/// Legal transitions follow the BLD.4 pipeline:
///   Init -> ConstructionDone -> Active -> Destroying -> Sinking -> FinalTeardown
/// Returns true if the transition was valid and applied.
pub fn transition_building_state(building: &mut BuildingData, new_state: BuildingState) -> bool {
    let valid = match (building.state, new_state) {
        (BuildingState::Init, BuildingState::ConstructionDone) => true,
        (BuildingState::ConstructionDone, BuildingState::Active) => true,
        (BuildingState::Active, BuildingState::Destroying) => true,
        (BuildingState::Destroying, BuildingState::Sinking) => true,
        (BuildingState::Sinking, BuildingState::FinalTeardown) => true,
        _ => false,
    };
    if valid {
        building.state = new_state;
    }
    valid
}

/// Called when a building completes construction (state transitions to Active).
/// Sets behavior_flags based on subtype from the type properties table (BLD.5).
pub fn on_construction_complete(building: &mut BuildingData) {
    building.behavior_flags = building_behavior_flags(building.building_subtype);
}

/// Called when a building is destroyed.
/// Clears all occupants and sets damage to max.
pub fn on_destroy(building: &mut BuildingData) {
    for slot in building.occupant_slots.iter_mut() {
        *slot = None;
    }
    building.occupant_count = 0;
    building.damage_accumulated = building_max_health(building.building_subtype);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_transition_init_to_construction_done() {
        let mut b = BuildingData::default();
        assert_eq!(b.state, BuildingState::Init);
        assert!(transition_building_state(&mut b, BuildingState::ConstructionDone));
        assert_eq!(b.state, BuildingState::ConstructionDone);
    }

    #[test]
    fn valid_transition_construction_done_to_active() {
        let mut b = BuildingData::default();
        b.state = BuildingState::ConstructionDone;
        assert!(transition_building_state(&mut b, BuildingState::Active));
        assert_eq!(b.state, BuildingState::Active);
    }

    #[test]
    fn valid_transition_active_to_destroying() {
        let mut b = BuildingData::default();
        b.state = BuildingState::Active;
        assert!(transition_building_state(&mut b, BuildingState::Destroying));
        assert_eq!(b.state, BuildingState::Destroying);
    }

    #[test]
    fn valid_transition_destroying_to_sinking() {
        let mut b = BuildingData::default();
        b.state = BuildingState::Destroying;
        assert!(transition_building_state(&mut b, BuildingState::Sinking));
        assert_eq!(b.state, BuildingState::Sinking);
    }

    #[test]
    fn valid_transition_sinking_to_final_teardown() {
        let mut b = BuildingData::default();
        b.state = BuildingState::Sinking;
        assert!(transition_building_state(&mut b, BuildingState::FinalTeardown));
        assert_eq!(b.state, BuildingState::FinalTeardown);
    }

    #[test]
    fn invalid_transition_init_to_active() {
        let mut b = BuildingData::default();
        assert!(!transition_building_state(&mut b, BuildingState::Active));
        assert_eq!(b.state, BuildingState::Init); // unchanged
    }

    #[test]
    fn invalid_transition_active_to_init() {
        let mut b = BuildingData::default();
        b.state = BuildingState::Active;
        assert!(!transition_building_state(&mut b, BuildingState::Init));
        assert_eq!(b.state, BuildingState::Active); // unchanged
    }

    #[test]
    fn invalid_transition_skip_states() {
        let mut b = BuildingData::default();
        assert!(!transition_building_state(&mut b, BuildingState::Destroying));
        assert_eq!(b.state, BuildingState::Init);
    }

    #[test]
    fn on_construction_complete_sets_flags() {
        let mut b = BuildingData::default();
        b.building_subtype = BuildingSubtype::SmallHut;
        on_construction_complete(&mut b);
        assert_eq!(b.behavior_flags, 0x20); // housing flag

        b.building_subtype = BuildingSubtype::WarriorTrain;
        on_construction_complete(&mut b);
        assert_eq!(b.behavior_flags, 0x01); // training flag
    }

    #[test]
    fn on_destroy_clears_occupants_and_maxes_damage() {
        let mut b = BuildingData::default();
        b.building_subtype = BuildingSubtype::SmallHut;
        b.occupant_slots[0] = Some(42);
        b.occupant_slots[1] = Some(99);
        b.occupant_count = 2;
        on_destroy(&mut b);
        assert_eq!(b.occupant_count, 0);
        for slot in &b.occupant_slots {
            assert!(slot.is_none());
        }
        assert_eq!(b.damage_accumulated, 600); // SmallHut max health
    }
}
